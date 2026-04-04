import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import { expect } from "chai";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAccount as createLegacyTokenAccount,
  createAssociatedTokenAccountInstruction,
  createMint as createLegacyMint,
  getAccount,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import {
  AccountMeta,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
} from "@solana/web3.js";

const METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
);

const GAME_CONFIG_SEED = Buffer.from("game-config");
const PLAYER_SEED = Buffer.from("player");
const RESOURCE_MINT_SEED = Buffer.from("resource-mint");
const RESOURCE_AUTHORITY_SEED = Buffer.from("resource-authority");
const MAGIC_MINT_SEED = Buffer.from("magic-mint");
const MAGIC_AUTHORITY_SEED = Buffer.from("magic-authority");
const SEARCH_AUTHORITY_SEED = Buffer.from("search-authority");
const CRAFTING_AUTHORITY_SEED = Buffer.from("crafting-authority");
const MARKETPLACE_AUTHORITY_SEED = Buffer.from("marketplace-authority");
const ITEM_AUTHORITY_SEED = Buffer.from("item-authority");
const ITEM_MINT_SEED = Buffer.from("item-mint");
const ITEM_METADATA_SEED = Buffer.from("item-metadata");

const ITEM_PRICES = [25, 45, 60, 90];
const TEST_ITEM_TYPE = 0;
const TEST_RECIPE = [1, 3, 0, 1, 0, 0];

const RESOURCE_DEFS = [
  {
    name: "Wood",
    symbol: "WOOD",
    uri: "https://example.com/resources/wood.json",
  },
  {
    name: "Iron",
    symbol: "IRON",
    uri: "https://example.com/resources/iron.json",
  },
  {
    name: "Gold",
    symbol: "GOLD",
    uri: "https://example.com/resources/gold.json",
  },
  {
    name: "Leather",
    symbol: "LEATHER",
    uri: "https://example.com/resources/leather.json",
  },
  {
    name: "Stone",
    symbol: "STONE",
    uri: "https://example.com/resources/stone.json",
  },
  {
    name: "Diamond",
    symbol: "DIAMOND",
    uri: "https://example.com/resources/diamond.json",
  },
];
const INVALID_RESOURCE_ID = RESOURCE_DEFS.length;
const MAGIC_TOKEN_DEF = {
  name: "Magic Token",
  symbol: "MAGIC",
  uri: "https://example.com/resources/magic.json",
};

export function createSuiteContext() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const payer = (
    provider.wallet as anchor.Wallet & { payer: Keypair }
  ).payer;

  const resourceManager: any = anchor.workspace.ResourceManager;
  const magicToken: any = anchor.workspace.MagicToken;
  const itemNft: any = anchor.workspace.ItemNft;
  const searchProgram: any = anchor.workspace.Search;
  const crafting: any = anchor.workspace.Crafting;
  const marketplace: any = anchor.workspace.Marketplace;

  const [gameConfig] = PublicKey.findProgramAddressSync(
    [GAME_CONFIG_SEED],
    resourceManager.programId,
  );
  const [resourceAuthority] = PublicKey.findProgramAddressSync(
    [RESOURCE_AUTHORITY_SEED, gameConfig.toBuffer()],
    resourceManager.programId,
  );
  const [searchAuthority] = PublicKey.findProgramAddressSync(
    [SEARCH_AUTHORITY_SEED, gameConfig.toBuffer()],
    searchProgram.programId,
  );
  const [craftingAuthority] = PublicKey.findProgramAddressSync(
    [CRAFTING_AUTHORITY_SEED, gameConfig.toBuffer()],
    crafting.programId,
  );
  const [marketplaceAuthority] = PublicKey.findProgramAddressSync(
    [MARKETPLACE_AUTHORITY_SEED, gameConfig.toBuffer()],
    marketplace.programId,
  );
  const [itemAuthority] = PublicKey.findProgramAddressSync(
    [ITEM_AUTHORITY_SEED, gameConfig.toBuffer()],
    itemNft.programId,
  );
  const [magicMint] = PublicKey.findProgramAddressSync(
    [MAGIC_MINT_SEED, gameConfig.toBuffer()],
    magicToken.programId,
  );
  const [magicAuthority] = PublicKey.findProgramAddressSync(
    [MAGIC_AUTHORITY_SEED, gameConfig.toBuffer()],
    magicToken.programId,
  );
  const resourceMints = RESOURCE_DEFS.map((_, index) =>
    PublicKey.findProgramAddressSync(
      [RESOURCE_MINT_SEED, gameConfig.toBuffer(), Buffer.from([index])],
      resourceManager.programId,
    )[0],
  );

  async function initialize() {
    await resourceManager.methods
      .initializeGame(ITEM_PRICES.map((value) => new BN(value)))
      .accounts({
        admin: payer.publicKey,
        gameConfig,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    for (const [index, resource] of RESOURCE_DEFS.entries()) {
      await resourceManager.methods
        .initializeResourceMint(
          index,
          resource.name,
          resource.symbol,
          resource.uri,
        )
        .accounts({
          admin: payer.publicKey,
          gameConfig,
          mint: resourceMints[index],
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    }

    await magicToken.methods
      .initializeMagicToken(
        MAGIC_TOKEN_DEF.name,
        MAGIC_TOKEN_DEF.symbol,
        MAGIC_TOKEN_DEF.uri,
      )
      .accounts({
        admin: payer.publicKey,
        gameConfig,
        magicMint,
        mintAuthority: magicAuthority,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    await resourceManager.methods
      .registerMagicTokenMint()
      .accounts({
        admin: payer.publicKey,
        gameConfig,
        magicTokenMint: magicMint,
      })
      .rpc();
  }

  async function fundUser(user: Keypair, sol: number) {
    const transaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: user.publicKey,
        lamports: Math.floor(sol * LAMPORTS_PER_SOL),
      }),
    );
    await provider.sendAndConfirm(transaction, []);
  }

  function playerAddress(owner: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [PLAYER_SEED, owner.toBuffer()],
      searchProgram.programId,
    );
  }

  function resourceTokenAccounts(owner: PublicKey): PublicKey[] {
    return resourceMints.map((mint) =>
      getAssociatedTokenAddressSync(
        mint,
        owner,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
      ),
    );
  }

  function recipeRemainingAccounts(owner: PublicKey, recipe: number[]): AccountMeta[] {
    const tokenAccounts = resourceTokenAccounts(owner);
    const metas: AccountMeta[] = [];
    for (const [index, amount] of recipe.entries()) {
      if (amount === 0) {
        continue;
      }
      metas.push({
        pubkey: resourceMints[index],
        isSigner: false,
        isWritable: true,
      });
      metas.push({
        pubkey: tokenAccounts[index],
        isSigner: false,
        isWritable: true,
      });
    }
    return metas;
  }

  function deriveItemAddresses(owner: PublicKey, mintSeed: number[]) {
    const [itemMint] = PublicKey.findProgramAddressSync(
      [ITEM_MINT_SEED, owner.toBuffer(), Buffer.from(mintSeed)],
      itemNft.programId,
    );
    const [itemMetadata] = PublicKey.findProgramAddressSync(
      [ITEM_METADATA_SEED, itemMint.toBuffer()],
      itemNft.programId,
    );
    const ownerItemAta = getAssociatedTokenAddressSync(itemMint, owner, false);
    const [metadataPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("metadata"), METADATA_PROGRAM_ID.toBuffer(), itemMint.toBuffer()],
      METADATA_PROGRAM_ID,
    );
    const [masterEditionPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        itemMint.toBuffer(),
        Buffer.from("edition"),
      ],
      METADATA_PROGRAM_ID,
    );

    return {
      itemMint,
      itemMetadata,
      ownerItemAta,
      metadataPda,
      masterEditionPda,
    };
  }

  function directMintItemContext(
    player: PublicKey,
    mintSeed: number[],
    craftingAuthoritySigner: PublicKey,
  ) {
    const {
      itemMint,
      itemMetadata,
      ownerItemAta,
      metadataPda,
      masterEditionPda,
    } = deriveItemAddresses(player, mintSeed);
    return {
      itemMint,
      itemMetadata,
      ownerItemAta,
      metadataPda,
      masterEditionPda,
      accounts: {
        player,
        gameConfig,
        craftingAuthority: craftingAuthoritySigner,
        itemAuthority,
        itemMint,
        playerItemAccount: ownerItemAta,
        itemMetadata,
        metadata: metadataPda,
        masterEdition: masterEditionPda,
        metadataProgram: METADATA_PROGRAM_ID,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      },
    };
  }

  function searchAccounts(owner: PublicKey, player: PublicKey) {
    const tokenAccounts = resourceTokenAccounts(owner);
    return {
      owner,
      player,
      gameConfig,
      searchAuthority,
      resourceAuthority,
      resourceManagerProgram: resourceManager.programId,
      woodMint: resourceMints[0],
      ironMint: resourceMints[1],
      goldMint: resourceMints[2],
      leatherMint: resourceMints[3],
      stoneMint: resourceMints[4],
      diamondMint: resourceMints[5],
      woodAccount: tokenAccounts[0],
      ironAccount: tokenAccounts[1],
      goldAccount: tokenAccounts[2],
      leatherAccount: tokenAccounts[3],
      stoneAccount: tokenAccounts[4],
      diamondAccount: tokenAccounts[5],
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    };
  }

  async function ensureAllResourceAtas(owner: PublicKey) {
    for (const mint of resourceMints) {
      await ensureToken2022Ata(mint, owner);
    }
  }

  async function ensureToken2022Ata(mint: PublicKey, owner: PublicKey) {
    const ata = getAssociatedTokenAddressSync(
      mint,
      owner,
      false,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    try {
      await getAccount(
        provider.connection,
        ata,
        "confirmed",
        TOKEN_2022_PROGRAM_ID,
      );
      return ata;
    } catch {
      const transaction = new Transaction().add(
        createAssociatedTokenAccountInstruction(
          payer.publicKey,
          ata,
          owner,
          mint,
          TOKEN_2022_PROGRAM_ID,
          ASSOCIATED_TOKEN_PROGRAM_ID,
        ),
      );
      await provider.sendAndConfirm(transaction, []);
      return ata;
    }
  }

  async function expectAllAccountsExist(
    accounts: PublicKey[],
    tokenProgram = TOKEN_2022_PROGRAM_ID,
  ) {
    await Promise.all(
      accounts.map((account) =>
        getAccount(provider.connection, account, "confirmed", tokenProgram),
      ),
    );
  }

  async function createLegacyMintForTests() {
    return createLegacyMint(provider.connection, payer, payer.publicKey, null, 0);
  }

  async function createLegacyTokenAccountForOwner(mint: PublicKey, owner: PublicKey) {
    return createLegacyTokenAccount(
      provider.connection,
      payer,
      mint,
      owner,
      Keypair.generate(),
    );
  }

  async function loadBalances(accounts: PublicKey[]) {
    return Promise.all(
      accounts.map(async (account) => {
        try {
          const tokenAccount = await getAccount(
            provider.connection,
            account,
            "confirmed",
            TOKEN_2022_PROGRAM_ID,
          );
          return Number(tokenAccount.amount);
        } catch {
          return 0;
        }
      }),
    );
  }

  async function sweepCollectorToCrafter(collector: Keypair, crafter: PublicKey) {
    const sourceAccounts = resourceTokenAccounts(collector.publicKey);
    const destinationAccounts = resourceTokenAccounts(crafter);
    await waitForBalances(
      sourceAccounts,
      (balances) => balances.reduce((sum, value) => sum + value, 0) === 3,
    );

    for (let index = 0; index < resourceMints.length; index += 1) {
      const source = await getAccount(
        provider.connection,
        sourceAccounts[index],
        "confirmed",
        TOKEN_2022_PROGRAM_ID,
      );
      if (source.amount === 0n) {
        continue;
      }

      await resourceManager.methods
        .transferResource(index, new BN(source.amount.toString()))
        .accounts({
          owner: collector.publicKey,
          gameConfig,
          mint: resourceMints[index],
          source: sourceAccounts[index],
          destination: destinationAccounts[index],
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([collector])
        .rpc();
    }
  }

  async function hasRecipe(accounts: PublicKey[], recipe: number[]) {
    const balances = await loadBalances(accounts);
    return recipe.every((needed, index) => balances[index] >= needed);
  }

  async function waitForBalances(
    accounts: PublicKey[],
    predicate: (balances: number[]) => boolean,
    attempts = 20,
    delayMs = 250,
  ) {
    let lastBalances: number[] = [];
    for (let attempt = 0; attempt < attempts; attempt += 1) {
      lastBalances = await loadBalances(accounts);
      if (predicate(lastBalances)) {
        return lastBalances;
      }
      await sleep(delayMs);
    }
    return lastBalances;
  }

  async function waitForStableBalances(
    accounts: PublicKey[],
    predicate: (balances: number[]) => boolean,
    attempts = 40,
    delayMs = 250,
    stableReadsRequired = 3,
  ) {
    let lastBalances: number[] = [];
    let stableReads = 0;

    for (let attempt = 0; attempt < attempts; attempt += 1) {
      const balances = await loadBalances(accounts);
      if (predicate(balances)) {
        if (
          lastBalances.length === balances.length &&
          balances.every((value, index) => value === lastBalances[index])
        ) {
          stableReads += 1;
        } else {
          stableReads = 1;
        }
        if (stableReads >= stableReadsRequired) {
          return balances;
        }
      } else {
        stableReads = 0;
      }
      lastBalances = balances;
      await sleep(delayMs);
    }

    return lastBalances;
  }

  async function waitForTokenBalance(
    account: PublicKey,
    amount: number,
    tokenProgram = anchor.utils.token.TOKEN_PROGRAM_ID,
    attempts = 20,
    delayMs = 250,
  ) {
    let lastError: unknown;
    for (let attempt = 0; attempt < attempts; attempt += 1) {
      try {
        const tokenAccount = await getAccount(
          provider.connection,
          account,
          "confirmed",
          tokenProgram,
        );
        if (Number(tokenAccount.amount) === amount) {
          return tokenAccount;
        }
      } catch (error) {
        lastError = error;
      }
      await sleep(delayMs);
    }
    if (lastError) {
      throw lastError;
    }
    return getAccount(provider.connection, account, "confirmed", tokenProgram);
  }

  async function expectZeroOrClosedTokenAccount(
    account: PublicKey,
    tokenProgram = anchor.utils.token.TOKEN_PROGRAM_ID,
    attempts = 20,
    delayMs = 250,
  ) {
    for (let attempt = 0; attempt < attempts; attempt += 1) {
      try {
        const tokenAccount = await getAccount(
          provider.connection,
          account,
          "confirmed",
          tokenProgram,
        );
        if (Number(tokenAccount.amount) === 0) {
          return;
        }
      } catch {
        return;
      }
      await sleep(delayMs);
    }

    const tokenAccount = await getAccount(
      provider.connection,
      account,
      "confirmed",
      tokenProgram,
    );
    expect(Number(tokenAccount.amount)).to.equal(0);
  }

  async function sleep(ms: number) {
    await new Promise((resolve) => setTimeout(resolve, ms));
  }

  async function expectReject(promise: Promise<unknown>) {
    let didReject = false;
    try {
      await promise;
    } catch {
      didReject = true;
    }
    expect(didReject).to.equal(true);
  }

  return {
    provider,
    payer,
    resourceManager,
    magicToken,
    itemNft,
    searchProgram,
    crafting,
    marketplace,
    gameConfig,
    resourceAuthority,
    searchAuthority,
    craftingAuthority,
    marketplaceAuthority,
    itemAuthority,
    magicMint,
    magicAuthority,
    resourceMints,
    metadataProgramId: METADATA_PROGRAM_ID,
    itemPrices: ITEM_PRICES,
    testItemType: TEST_ITEM_TYPE,
    testRecipe: TEST_RECIPE,
    resourceDefs: RESOURCE_DEFS,
    invalidResourceId: INVALID_RESOURCE_ID,
    magicTokenDef: MAGIC_TOKEN_DEF,
    initialize,
    fundUser,
    playerAddress,
    resourceTokenAccounts,
    recipeRemainingAccounts,
    deriveItemAddresses,
    directMintItemContext,
    searchAccounts,
    ensureAllResourceAtas,
    ensureToken2022Ata,
    expectAllAccountsExist,
    createLegacyMintForTests,
    createLegacyTokenAccountForOwner,
    loadBalances,
    sweepCollectorToCrafter,
    hasRecipe,
    waitForBalances,
    waitForStableBalances,
    waitForTokenBalance,
    expectZeroOrClosedTokenAccount,
    expectReject,
  };
}

export type SuiteContext = ReturnType<typeof createSuiteContext>;
