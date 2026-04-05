/** Integration tests for recipe validation, resource burning and item minting. */
import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { createRequire } from "module";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotent,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import BN from "bn.js";

const require = createRequire(`${process.cwd()}/tests/crafting.ts`);
const { getGamePrograms, getProgramPublicKey } = require("../utils/programs");
const {
  findGameConfigPda,
  findItemMetadataPda,
  findMagicTokenMintPda,
  findPlayerPda,
  findProgramAuthorityPda,
  findResourceMintPda,
} = require("../utils/account_utils");

/** Canonical Metaplex Token Metadata program used by item NFTs. */
const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
);

/** Static resource metadata used while bootstrapping test mints. */
const RESOURCE_METADATA = [
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
    symbol: "LETHR",
    uri: "https://example.com/resources/leather.json",
  },
  {
    name: "Stone",
    symbol: "STONE",
    uri: "https://example.com/resources/stone.json",
  },
  {
    name: "Diamond",
    symbol: "DIAM",
    uri: "https://example.com/resources/diamond.json",
  },
] as const;

/** Canonical recipes expected by the crafting program. */
const RECIPES = [
  {
    itemType: 0,
    label: "Kozak Sabre",
    costs: [1, 3, 0, 1, 0, 0],
  },
  {
    itemType: 1,
    label: "Elder Staff",
    costs: [2, 0, 1, 0, 0, 1],
  },
  {
    itemType: 2,
    label: "Characteristic Armor",
    costs: [0, 2, 1, 4, 0, 0],
  },
  {
    itemType: 3,
    label: "Battle Bracelet",
    costs: [0, 4, 2, 0, 0, 2],
  },
] as const;

describe("crafting", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programs = getGamePrograms(provider);
  const admin = (provider.wallet as anchor.Wallet).payer;
  const itemPrices = [25, 40, 75, 110].map((value) => new BN(value));
  const [gameConfigPda] = findGameConfigPda();
  const [magicTokenMintPda] = findMagicTokenMintPda();
  const [searchAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("search"),
  );
  const [resourceManagerAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("resource_manager"),
  );
  const [craftingAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("crafting"),
  );
  const [itemNftAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("item_nft"),
  );
  const craftComputeBudgetIx =
    anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 500_000,
    });

  const resourceMints = RESOURCE_METADATA.map(
    (_resource, resourceType) => findResourceMintPda(resourceType)[0],
  );

  /** Asserts that a transaction promise fails without caring about the exact error text. */
  const expectRpcToFail = async (promise: Promise<unknown>) => {
    try {
      await promise;
      expect.fail("Expected transaction to fail");
    } catch (_error) {
      expect(true).to.equal(true);
    }
  };

  /** Derives the Metaplex metadata PDA for an NFT mint. */
  const deriveMetadataPda = (
    mint: anchor.web3.PublicKey,
  ): anchor.web3.PublicKey => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID,
    )[0];
  };

  /** Derives the Metaplex master edition PDA for an NFT mint. */
  const deriveMasterEditionPda = (
    mint: anchor.web3.PublicKey,
  ): anchor.web3.PublicKey => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
        Buffer.from("edition"),
      ],
      TOKEN_METADATA_PROGRAM_ID,
    )[0];
  };

  /** Ensures a test wallet has enough SOL for minting and token-account creation. */
  const ensureWalletFunded = async (wallet: anchor.web3.Keypair) => {
    const balance = await provider.connection.getBalance(
      wallet.publicKey,
      "confirmed",
    );
    if (balance >= anchor.web3.LAMPORTS_PER_SOL / 2) {
      return;
    }

    const signature = await provider.connection.requestAirdrop(
      wallet.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(signature, "confirmed");
  };

  /** Creates the shared config and all canonical resource mints when missing. */
  const ensureBootstrap = async () => {
    const existingGameConfig =
      await programs.resource_manager.account.gameConfig.fetchNullable(
        gameConfigPda,
      );

    if (!existingGameConfig) {
      await programs.resource_manager.methods
        .initializeGameConfig(magicTokenMintPda, itemPrices)
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin])
        .rpc();
    }

    for (
      let resourceType = 0;
      resourceType < RESOURCE_METADATA.length;
      resourceType += 1
    ) {
      const resourceMintPda = resourceMints[resourceType];
      const accountInfo = await provider.connection.getAccountInfo(
        resourceMintPda,
        "confirmed",
      );

      if (accountInfo) {
        continue;
      }

      const metadata = RESOURCE_METADATA[resourceType];
      await programs.resource_manager.methods
        .initializeResourceMint(
          resourceType,
          metadata.name,
          metadata.symbol,
          metadata.uri,
        )
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          programAuthority: resourceManagerAuthority,
          resourceMint: resourceMintPda,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin])
        .rpc();
    }
  };

  /** Creates the search-player account for a wallet if it does not exist yet. */
  const ensurePlayerInitialized = async (player: anchor.web3.Keypair) => {
    const [playerPda] = findPlayerPda(player.publicKey);
    const existingPlayer =
      await programs.search.account.player.fetchNullable(playerPda);

    if (!existingPlayer) {
      await programs.search.methods
        .initPlayer()
        .accounts({
          owner: player.publicKey,
          player: playerPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([player])
        .rpc();
    }

    return playerPda as anchor.web3.PublicKey;
  };

  /** Creates and returns the player's ATA for a specific resource mint. */
  const ensurePlayerResourceAta = async (
    player: anchor.web3.PublicKey,
    resourceType: number,
  ): Promise<anchor.web3.PublicKey> => {
    const ata = getAssociatedTokenAddressSync(
      resourceMints[resourceType],
      player,
      false,
      TOKEN_2022_PROGRAM_ID,
    );

    await createAssociatedTokenAccountIdempotent(
      provider.connection,
      admin,
      resourceMints[resourceType],
      player,
      {},
      TOKEN_2022_PROGRAM_ID,
    );

    return ata;
  };

  /** Mints one resource amount to the player through the authorized search CPI. */
  const mintResourceToPlayer = async (
    player: anchor.web3.Keypair,
    playerPda: anchor.web3.PublicKey,
    resourceType: number,
    amount: number,
  ) => {
    if (amount === 0) {
      return;
    }

    const playerAta = await ensurePlayerResourceAta(
      player.publicKey,
      resourceType,
    );
    const signature = await programs.search.methods
      .proxyMintResource(resourceType, new BN(amount))
      .accounts({
        owner: player.publicKey,
        player: playerPda,
        gameConfig: gameConfigPda,
        searchAuthority,
        resourceManagerAuthority,
        resourceMint: resourceMints[resourceType],
        playerResourceTokenAccount: playerAta,
        resourceManagerProgram: getProgramPublicKey("resource_manager"),
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player])
      .rpc();

    await provider.connection.confirmTransaction(signature, "confirmed");
  };

  /** Seeds all resource balances required by one recipe. */
  const seedRecipeResources = async (
    player: anchor.web3.Keypair,
    playerPda: anchor.web3.PublicKey,
    costs: readonly number[],
  ) => {
    for (let resourceType = 0; resourceType < costs.length; resourceType += 1) {
      await mintResourceToPlayer(
        player,
        playerPda,
        resourceType,
        costs[resourceType],
      );
    }
  };

  /** Reads one resource balance for the given player wallet. */
  const readResourceBalance = async (
    player: anchor.web3.PublicKey,
    resourceType: number,
  ): Promise<bigint> => {
    const tokenAccount = getAssociatedTokenAddressSync(
      resourceMints[resourceType],
      player,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
    const accountInfo = await provider.connection.getAccountInfo(
      tokenAccount,
      "confirmed",
    );

    if (!accountInfo) {
      return 0n;
    }

    const account = await getAccount(
      provider.connection,
      tokenAccount,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );
    return account.amount;
  };

  /** Creates and funds a fresh player session with all resource ATAs ready. */
  const createPlayerSession = async () => {
    const player = anchor.web3.Keypair.generate();
    await ensureWalletFunded(player);
    const playerPda = await ensurePlayerInitialized(player);
    const resourceTokenAccounts = await Promise.all(
      RESOURCE_METADATA.map((_resource, resourceType) =>
        ensurePlayerResourceAta(player.publicKey, resourceType),
      ),
    );

    return {
      player,
      playerPda,
      resourceTokenAccounts,
    };
  };

  /** Builds the fixed account map needed by the `craftItem` instruction. */
  const buildCraftAccounts = (
    owner: anchor.web3.PublicKey,
    mint: anchor.web3.PublicKey,
  ) => {
    const [itemMetadataPda] = findItemMetadataPda(mint);
    const metadataPda = deriveMetadataPda(mint);
    const masterEditionPda = deriveMasterEditionPda(mint);
    const ownerItemTokenAccount = getAssociatedTokenAddressSync(
      mint,
      owner,
      false,
      TOKEN_PROGRAM_ID,
    );

    return {
      itemMetadataPda,
      metadataPda,
      masterEditionPda,
      ownerItemTokenAccount,
      accounts: {
        owner,
        gameConfig: gameConfigPda,
        craftingAuthority,
        itemNftAuthority,
        mint,
        itemMetadata: itemMetadataPda,
        metadata: metadataPda,
        masterEdition: masterEditionPda,
        ownerItemTokenAccount,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        resourceManagerProgram: getProgramPublicKey("resource_manager"),
        itemNftProgram: getProgramPublicKey("item_nft"),
        resourceTokenProgram: TOKEN_2022_PROGRAM_ID,
        itemTokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      },
    };
  };

  /** Builds the ordered remaining-account pairs expected by the crafting program. */
  const buildCraftRemainingAccounts = (
    recipeCosts: readonly number[],
    resourceTokenAccounts: readonly anchor.web3.PublicKey[],
  ) => {
    return recipeCosts.flatMap((cost, resourceType) => {
      if (cost === 0) {
        return [];
      }

      return [
        {
          pubkey: resourceMints[resourceType],
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: resourceTokenAccounts[resourceType],
          isWritable: true,
          isSigner: false,
        },
      ];
    });
  };

  before(async () => {
    await ensureBootstrap();
  });

  for (const recipe of RECIPES) {
    it(`crafts ${recipe.label} by burning the exact recipe resources`, async () => {
      const session = await createPlayerSession();
      await seedRecipeResources(
        session.player,
        session.playerPda,
        recipe.costs,
      );

      const resourceBalancesBefore = await Promise.all(
        RESOURCE_METADATA.map((_resource, resourceType) =>
          readResourceBalance(session.player.publicKey, resourceType),
        ),
      );

      const mint = anchor.web3.Keypair.generate();
      const craftedItem = buildCraftAccounts(
        session.player.publicKey,
        mint.publicKey,
      );
      const remainingAccounts = buildCraftRemainingAccounts(
        recipe.costs,
        session.resourceTokenAccounts,
      );

      const signature = await programs.crafting.methods
        .craftItem(recipe.itemType)
        .preInstructions([craftComputeBudgetIx])
        .accounts(craftedItem.accounts)
        .remainingAccounts(remainingAccounts)
        .signers([session.player, mint])
        .rpc();

      await provider.connection.confirmTransaction(signature, "confirmed");

      const resourceBalancesAfter = await Promise.all(
        RESOURCE_METADATA.map((_resource, resourceType) =>
          readResourceBalance(session.player.publicKey, resourceType),
        ),
      );
      const mintAccount = await getMint(
        provider.connection,
        mint.publicKey,
        "confirmed",
        TOKEN_PROGRAM_ID,
      );
      const ownerItemAccount = await getAccount(
        provider.connection,
        craftedItem.ownerItemTokenAccount,
        "confirmed",
        TOKEN_PROGRAM_ID,
      );
      const itemMetadata = await programs.item_nft.account.itemMetadata.fetch(
        craftedItem.itemMetadataPda,
      );

      for (
        let resourceType = 0;
        resourceType < recipe.costs.length;
        resourceType += 1
      ) {
        expect(
          resourceBalancesBefore[resourceType] -
            resourceBalancesAfter[resourceType],
        ).to.equal(BigInt(recipe.costs[resourceType]));
      }

      expect(Number(mintAccount.supply)).to.equal(1);
      expect(mintAccount.decimals).to.equal(0);
      expect(Number(ownerItemAccount.amount)).to.equal(1);
      expect(itemMetadata.itemType).to.equal(recipe.itemType);
      expect(itemMetadata.owner.toBase58()).to.equal(
        session.player.publicKey.toBase58(),
      );
      expect(itemMetadata.mint.toBase58()).to.equal(mint.publicKey.toBase58());
    });
  }

  it("rejects crafting when at least one required resource is missing", async () => {
    const session = await createPlayerSession();
    const recipe = RECIPES[3];
    const underfundedCosts = [0, 4, 1, 0, 0, 2];

    await seedRecipeResources(
      session.player,
      session.playerPda,
      underfundedCosts,
    );

    const resourceBalancesBefore = await Promise.all(
      RESOURCE_METADATA.map((_resource, resourceType) =>
        readResourceBalance(session.player.publicKey, resourceType),
      ),
    );

    const mint = anchor.web3.Keypair.generate();
    const craftedItem = buildCraftAccounts(
      session.player.publicKey,
      mint.publicKey,
    );
    const remainingAccounts = buildCraftRemainingAccounts(
      recipe.costs,
      session.resourceTokenAccounts,
    );

    await expectRpcToFail(
      programs.crafting.methods
        .craftItem(recipe.itemType)
        .preInstructions([craftComputeBudgetIx])
        .accounts(craftedItem.accounts)
        .remainingAccounts(remainingAccounts)
        .signers([session.player, mint])
        .rpc(),
    );

    const resourceBalancesAfter = await Promise.all(
      RESOURCE_METADATA.map((_resource, resourceType) =>
        readResourceBalance(session.player.publicKey, resourceType),
      ),
    );
    const itemMetadataInfo = await provider.connection.getAccountInfo(
      craftedItem.itemMetadataPda,
      "confirmed",
    );

    expect(resourceBalancesAfter).to.deep.equal(resourceBalancesBefore);
    expect(itemMetadataInfo).to.equal(null);
  });
});
