import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createTransferInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  Signer,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
} from "@solana/web3.js";

import { Crafting } from "../../target/types/crafting";
import { ItemNft } from "../../target/types/item_nft";
import { MagicToken } from "../../target/types/magic_token";
import { Marketplace } from "../../target/types/marketplace";
import { ResourceManager } from "../../target/types/resource_manager";
import { Search } from "../../target/types/search";

export const resourceNames = ["wood", "iron", "gold", "leather", "stone", "diamond"] as const;

export type TestState = {
  provider: anchor.AnchorProvider;
  wallet: anchor.Wallet;
  playerAuthority: Keypair;
  resourceManager: Program<ResourceManager>;
  itemNft: Program<ItemNft>;
  crafting: Program<Crafting>;
  search: Program<Search>;
  marketplace: Program<Marketplace>;
  magicToken: Program<MagicToken>;
  magicMint: PublicKey;
  resourceMints: PublicKey[];
  gameConfig: PublicKey;
  resourceAuthority: PublicKey;
  magicConfig: PublicKey;
  magicMintAuthority: PublicKey;
  playerPda: PublicKey;
  searchAuthority: PublicKey;
  craftingAuthority: PublicKey;
  playerResourceAtas: PublicKey[];
  itemMintAuthority: PublicKey;
  marketAuthority: PublicKey;
};

export type CraftedItem = {
  mint: Keypair;
  metadata: PublicKey;
  ata: PublicKey;
  itemType: number;
};

type SetupCache = {
  state: TestState;
  initialized: boolean;
};

const cache: SetupCache = {
  initialized: false,
  // @ts-expect-error filled in during setup
  state: null,
};

let inFlight: Promise<TestState> | null = null;

export const getState = async (): Promise<TestState> => {
  if (cache.initialized) {
    return cache.state;
  }

  if (inFlight) {
    return inFlight;
  }

  inFlight = (async () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const wallet = provider.wallet as anchor.Wallet;

    const resourceManager = anchor.workspace.ResourceManager as Program<ResourceManager>;
    const itemNft = anchor.workspace.ItemNft as Program<ItemNft>;
    const crafting = anchor.workspace.Crafting as Program<Crafting>;
    const search = anchor.workspace.Search as Program<Search>;
    const marketplace = anchor.workspace.Marketplace as Program<Marketplace>;
    const magicToken = anchor.workspace.MagicToken as Program<MagicToken>;

    const playerAuthority = Keypair.generate();
    await airdrop(provider.connection, playerAuthority.publicKey);

    const [gameConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      resourceManager.programId,
    );
    const [resourceAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("resource_auth"), gameConfig.toBuffer()],
      resourceManager.programId,
    );

    const [magicConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_config")],
      magicToken.programId,
    );
    const [magicMintAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_auth")],
      magicToken.programId,
    );
    const [marketAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("market_exec")],
      marketplace.programId,
    );
    const [itemMintAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("item_mint_auth"), gameConfig.toBuffer()],
      itemNft.programId,
    );

    const payerKey = wallet.publicKey;
    const playerKey = playerAuthority.publicKey;
    const [playerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), playerKey.toBuffer()],
      search.programId,
    );
    const [searchAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("search"), playerKey.toBuffer()],
      search.programId,
    );
    const [craftingAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("craft"), playerKey.toBuffer()],
      crafting.programId,
    );

    const gameConfigInfo = await provider.connection.getAccountInfo(gameConfig);
    const magicConfigInfo = await provider.connection.getAccountInfo(magicConfig);

    if (gameConfigInfo && !magicConfigInfo) {
        throw new Error("MagicToken config missing but game config exists; reset local ledger.");
    }

    let magicMint: PublicKey;
    let resourceMints: PublicKey[] = [];

    if (!magicConfigInfo) {
      const magicMintKeypair = Keypair.generate();

      await magicToken.methods
        .initialize()
        .accountsStrict({
          payer: payerKey,
          admin: payerKey,
          marketplaceProgram: marketplace.programId,
          config: magicConfig,
          mint: magicMintKeypair.publicKey,
          mintAuthority: magicMintAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([magicMintKeypair])
        .rpc();

      magicMint = magicMintKeypair.publicKey;
    } else {
      const existingMint = await findMintByAuthority(provider.connection, magicMintAuthority);
      if (!existingMint) {
        throw new Error("Unable to locate MagicToken mint; reset local ledger.");
      }
      magicMint = existingMint;
    }

    if (!gameConfigInfo) {
      const resourceMintKeypairs = Array.from({ length: 6 }, () => Keypair.generate());
      const itemPrices = resourceNames
        .slice(0, 4)
        .map((_, idx) => new BN((idx + 1) * 5));

      await resourceManager.methods
        .initialize(
          itemPrices as [BN, BN, BN, BN],
          search.programId,
          crafting.programId,
          itemNft.programId,
          marketplace.programId,
        )
        .accountsStrict({
          payer: payerKey,
          admin: payerKey,
          magicTokenMint: magicMint,
          gameConfig,
          resourceAuthority,
          mintWood: resourceMintKeypairs[0].publicKey,
          mintIron: resourceMintKeypairs[1].publicKey,
          mintGold: resourceMintKeypairs[2].publicKey,
          mintLeather: resourceMintKeypairs[3].publicKey,
          mintStone: resourceMintKeypairs[4].publicKey,
          mintDiamond: resourceMintKeypairs[5].publicKey,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers(resourceMintKeypairs)
        .rpc();

      resourceMints = resourceMintKeypairs.map((kp) => kp.publicKey);
    } else {
      const configAccount = await resourceManager.account.gameConfig.fetch(gameConfig);
      magicMint = configAccount.magicTokenMint;
      resourceMints = configAccount.resourceMints as unknown as PublicKey[];
    }

    const playerResourceAtas = await ensureResourceAtas(
      provider,
      playerKey,
      resourceMints,
    );

    const playerAccountInfo = await provider.connection.getAccountInfo(playerPda);
    if (!playerAccountInfo || playerAccountInfo.owner.toBase58() != search.programId.toBase58()) {
      const initIx = await search.methods
        .initPlayer()
        .accountsStrict({
          payer: payerKey,
          owner: playerKey,
          player: playerPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([playerAuthority])
        .instruction();
      await provider.sendAndConfirm(new Transaction().add(initIx), [playerAuthority]);
    }

    cache.state = {
      provider,
      wallet,
      playerAuthority,
      resourceManager,
      itemNft,
      crafting,
      search,
      marketplace,
      magicToken,
      magicMint,
      resourceMints,
      gameConfig,
      resourceAuthority,
      magicConfig,
      magicMintAuthority,
      playerPda,
      searchAuthority,
      craftingAuthority,
      playerResourceAtas,
      itemMintAuthority,
      marketAuthority,
    };
    cache.initialized = true;
    return cache.state;
  })();

  try {
    return await inFlight;
  } finally {
    inFlight = null;
  }
};

export async function ensureResourceAtas(
  provider: anchor.AnchorProvider,
  owner: PublicKey,
  mints: PublicKey[],
): Promise<PublicKey[]> {
  const ixs = mints.map((mint) =>
    createAssociatedTokenAccountIdempotentInstruction(
      provider.wallet.publicKey,
      getAssociatedTokenAddressSync(mint, owner, false, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID),
      owner,
      mint,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    ),
  );

  if (ixs.length > 0) {
    const tx = new Transaction().add(...ixs);
    await provider.sendAndConfirm(tx, []);
  }

  return mints.map((mint) =>
    getAssociatedTokenAddressSync(mint, owner, false, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID),
  );
}

export async function getResourceBalances(
  provider: anchor.AnchorProvider,
  atas: PublicKey[],
): Promise<number[]> {
  return Promise.all(
    atas.map(async (ata) => {
      const account = await getAccount(provider.connection, ata, undefined, TOKEN_2022_PROGRAM_ID);
      return Number(account.amount);
    }),
  );
}

async function sendInstructions(
  provider: anchor.AnchorProvider,
  instructions: anchor.web3.TransactionInstruction[],
  signers: Signer[] = [],
) {
  if (instructions.length === 0) {
    return;
  }
  const tx = new Transaction().add(...instructions);
  await provider.sendAndConfirm(tx, signers);
}

export async function airdrop(
  connection: anchor.web3.Connection,
  pubkey: PublicKey,
  lamports = 2 * anchor.web3.LAMPORTS_PER_SOL,
) {
  const sig = await connection.requestAirdrop(pubkey, lamports);
  await connection.confirmTransaction(sig, "confirmed");
}

export async function farmResources(state: TestState, minBalances: number[]) {
  const { provider, search, resourceMints, resourceAuthority, gameConfig } = state;
  const connection = provider.connection;
  const ownerAtas = state.playerResourceAtas;
  let balances = await getResourceBalances(provider, ownerAtas);

  const needsMore = () => balances.some((bal, idx) => bal < minBalances[idx]);
  let attempts = 0;

  while (needsMore()) {
    attempts += 1;
    const helper = Keypair.generate();
    await airdrop(connection, helper.publicKey);

    const [helperPlayer] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), helper.publicKey.toBuffer()],
      search.programId,
    );
    const helperInitIx = await search.methods
      .initPlayer()
      .accountsStrict({
        payer: state.wallet.publicKey,
        owner: helper.publicKey,
        player: helperPlayer,
        systemProgram: SystemProgram.programId,
        })
      .signers([helper])
      .instruction();
    await provider.sendAndConfirm(new Transaction().add(helperInitIx), [helper]);

    const helperAtas = await ensureResourceAtas(
      provider,
      helper.publicKey,
      resourceMints,
    );

    const [helperSearchAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("search"), helper.publicKey.toBuffer()],
      search.programId,
    );

    await search.methods
      .searchResources()
      .accountsStrict({
        owner: helper.publicKey,
        player: helperPlayer,
        gameConfig,
        searchAuthority: helperSearchAuthority,
        resourceAuthority,
        mintWood: resourceMints[0],
        mintIron: resourceMints[1],
        mintGold: resourceMints[2],
        mintLeather: resourceMints[3],
        mintStone: resourceMints[4],
        mintDiamond: resourceMints[5],
        ataWood: helperAtas[0],
        ataIron: helperAtas[1],
        ataGold: helperAtas[2],
        ataLeather: helperAtas[3],
        ataStone: helperAtas[4],
        ataDiamond: helperAtas[5],
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        resourceManagerProgram: state.resourceManager.programId,
      })
      .signers([helper])
      .rpc();

    const transferIxs: anchor.web3.TransactionInstruction[] = [];
    const helperBalances = await getResourceBalances(provider, helperAtas);
    helperBalances.forEach((amount, idx) => {
      if (amount > 0) {
        transferIxs.push(
          createTransferInstruction(
            helperAtas[idx],
            ownerAtas[idx],
            helper.publicKey,
            BigInt(amount),
            [],
            TOKEN_2022_PROGRAM_ID,
          ),
        );
      }
    });

    await sendInstructions(provider, transferIxs, [helper]);
    balances = await getResourceBalances(provider, ownerAtas);

    if (attempts > 40) {
      throw new Error("Resource farming attempts exceeded safe threshold");
    }
  }
}

export async function getMagicMintInfo(state: TestState) {
  return getMint(state.provider.connection, state.magicMint, undefined, TOKEN_2022_PROGRAM_ID);
}

export const PROGRAM_IDS = {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
};

async function findMintByAuthority(connection: anchor.web3.Connection, authority: PublicKey) {
  const accounts = await connection.getProgramAccounts(TOKEN_2022_PROGRAM_ID, {
    filters: [
      {
        memcmp: {
          offset: 4,
          bytes: authority.toBase58(),
        },
      },
    ],
  });
  return accounts.length > 0 ? accounts[0].pubkey : null;
}

export async function craftItem(state: TestState, itemType: number): Promise<CraftedItem> {
  const { TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } = PROGRAM_IDS;
  const itemMint = Keypair.generate();
  const [metadata] = PublicKey.findProgramAddressSync(
    [Buffer.from("item_meta"), itemMint.publicKey.toBuffer()],
    state.itemNft.programId,
  );

  const ata = getAssociatedTokenAddressSync(
    itemMint.publicKey,
    state.playerAuthority.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  await state.crafting.methods
    .craft(itemType)
    .accountsStrict({
      payer: state.wallet.publicKey,
      player: state.playerAuthority.publicKey,
      craftingAuthority: state.craftingAuthority,
      gameConfig: state.gameConfig,
      itemMint: itemMint.publicKey,
      itemMetadata: metadata,
      mintAuthority: state.itemMintAuthority,
      playerItemAta: ata,
      mintWood: state.resourceMints[0],
      mintIron: state.resourceMints[1],
      mintGold: state.resourceMints[2],
      mintLeather: state.resourceMints[3],
      mintStone: state.resourceMints[4],
      mintDiamond: state.resourceMints[5],
      ataWood: state.playerResourceAtas[0],
      ataIron: state.playerResourceAtas[1],
      ataGold: state.playerResourceAtas[2],
      ataLeather: state.playerResourceAtas[3],
      ataStone: state.playerResourceAtas[4],
      ataDiamond: state.playerResourceAtas[5],
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
      resourceManagerProgram: state.resourceManager.programId,
      itemNftProgram: state.itemNft.programId,
    })
    .signers([itemMint, state.playerAuthority])
    .rpc();

  return { mint: itemMint, metadata, ata, itemType };
}
