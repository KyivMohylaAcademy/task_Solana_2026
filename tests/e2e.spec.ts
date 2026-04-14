/**
 * End-to-end integration test: full game loop.
 *
 * Sequence:
 *  1. Admin: initialize_config, create_resource_mint × 6, magic_token::initialize,
 *     set_magic_token_mint, item_nft::initialize_collection
 *  2. Player: register_player
 *  3. Player: run_search × 5 (clock warped +61s between each)
 *  4. Admin: top-up resources to exact recipe amounts (via mint_from_crafting CPI test helper)
 *  5. Player: craft all 4 items
 *  6. Player: sell all 4 items → assert MagicToken balance
 *
 * NOTE: This test requires `anchor build` to have completed so that the IDL files
 * and program binaries are available in target/.
 */
import * as anchor from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";
import { startAnchor } from "solana-bankrun";
import {
  Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY,
  SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
} from "@solana/web3.js";
import { expect } from "chai";
import {
  gameConfigPda, resourceAuthorityPda, resourceMintPda,
  searchAuthorityPda, craftingAuthorityPda, marketplaceAuthorityPda,
  magicMintPda, magicAuthorityPda, magicConfigPda,
  playerPda, itemMetadataPda, itemNftConfigPda, collectionAuthorityPda,
  getResourceAta, getMagicAta, advanceClock, PROGRAM_IDS,
} from "./helpers/setup";
import {
  TOKEN_2022_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  getAccount,
} from "@solana/spl-token";

const MPL_CORE_ID = new PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJe1bpb");

// Item recipes: [wood, iron, gold, leather, stone, diamond]
const RECIPES = [
  [1, 3, 0, 1, 0, 0], // Saber
  [2, 0, 1, 0, 0, 1], // Staff
  [0, 2, 1, 4, 0, 0], // Armor
  [0, 4, 2, 0, 0, 2], // Bracelet
];

describe("e2e: full game loop", () => {
  let context: any;
  let provider: BankrunProvider;
  let rmProgram: any;
  let mtProgram: any;
  let searchProgram: any;
  let itemNftProgram: any;
  let craftingProgram: any;
  let marketplaceProgram: any;
  let collectionKeypair: Keypair;

  // Resource mints are PDAs, not keypairs
  const resourceMints: PublicKey[] = [];

  before(async () => {
    collectionKeypair = Keypair.generate();
    context = await startAnchor(".", [], []);
    provider = new BankrunProvider(context);
    anchor.setProvider(provider as any);

    rmProgram = new anchor.Program(require("../target/idl/resource_manager.json"), provider as any);
    mtProgram = new anchor.Program(require("../target/idl/magic_token.json"), provider as any);
    searchProgram = new anchor.Program(require("../target/idl/search.json"), provider as any);
    itemNftProgram = new anchor.Program(require("../target/idl/item_nft.json"), provider as any);
    craftingProgram = new anchor.Program(require("../target/idl/crafting.json"), provider as any);
    marketplaceProgram = new anchor.Program(require("../target/idl/marketplace.json"), provider as any);

    for (let k = 0; k < 6; k++) {
      const [mint] = resourceMintPda(k);
      resourceMints.push(mint);
    }
  });

  // ── Phase 1: Admin initialization ──────────────────────────────────────────
  it("1a. initializes GameConfig", async () => {
    const [configPda] = gameConfigPda();
    await rmProgram.methods
      .initializeConfig([
        new anchor.BN(100), new anchor.BN(200),
        new anchor.BN(300), new anchor.BN(400),
      ])
      .accounts({
        admin: provider.wallet.publicKey,
        gameConfig: configPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const cfg = await rmProgram.account.gameConfig.fetch(configPda);
    expect(cfg.itemPrices[0].toNumber()).to.equal(100);
  });

  it("1b. creates all 6 resource mints", async () => {
    const [configPda] = gameConfigPda();
    const [resAuth] = resourceAuthorityPda();

    for (let kind = 0; kind < 6; kind++) {
      const [mintPda] = resourceMintPda(kind);
      await rmProgram.methods
        .createResourceMint(kind)
        .accounts({
          admin: provider.wallet.publicKey,
          gameConfig: configPda,
          mint: mintPda,
          resourceAuthority: resAuth,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .rpc();
    }

    const cfg = await rmProgram.account.gameConfig.fetch(configPda);
    for (let k = 0; k < 6; k++) {
      expect(cfg.resourceMints[k].toBase58()).to.not.equal(PublicKey.default.toBase58());
    }
  });

  it("1c. initializes MagicToken mint", async () => {
    const [magicMint] = magicMintPda();
    const [magicAuth] = magicAuthorityPda();
    const [magicCfg] = magicConfigPda();

    await mtProgram.methods
      .initialize()
      .accounts({
        admin: provider.wallet.publicKey,
        magicMint,
        magicAuthority: magicAuth,
        magicConfig: magicCfg,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    // Register in GameConfig
    const [configPda] = gameConfigPda();
    await rmProgram.methods
      .setMagicTokenMint(magicMint)
      .accounts({
        admin: provider.wallet.publicKey,
        gameConfig: configPda,
      })
      .rpc();

    const cfg = await rmProgram.account.gameConfig.fetch(configPda);
    expect(cfg.magicTokenMint.toBase58()).to.equal(magicMint.toBase58());
  });

  it("1d. initializes item NFT collection", async () => {
    const [colAuth] = collectionAuthorityPda();
    const [nftCfg] = itemNftConfigPda();

    await itemNftProgram.methods
      .initializeCollection()
      .accounts({
        admin: provider.wallet.publicKey,
        collection: collectionKeypair.publicKey,
        collectionAuthority: colAuth,
        itemNftConfig: nftCfg,
        mplCoreProgram: MPL_CORE_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([collectionKeypair])
      .rpc();

    const cfg = await itemNftProgram.account.itemNftConfig.fetch(nftCfg);
    expect(cfg.collection.toBase58()).to.equal(collectionKeypair.publicKey.toBase58());
  });

  // ── Phase 2: Player registration ───────────────────────────────────────────
  it("2. registers player", async () => {
    const [pda] = playerPda(provider.wallet.publicKey);
    await searchProgram.methods
      .registerPlayer()
      .accounts({
        owner: provider.wallet.publicKey,
        player: pda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const p = await searchProgram.account.player.fetch(pda);
    expect(p.searchNonce.toNumber()).to.equal(0);
  });

  // ── Phase 3: Resource search ────────────────────────────────────────────────
  it("3. searches 5 times (clock warped)", async () => {
    const [playerPdaAddr] = playerPda(provider.wallet.publicKey);
    const [gameConfig] = gameConfigPda();
    const [searchAuth] = searchAuthorityPda();
    const [resAuth] = resourceAuthorityPda();

    // Create ATAs for player
    const conn = provider.connection;
    for (let k = 0; k < 6; k++) {
      const mint = resourceMints[k];
      await getOrCreateAssociatedTokenAccount(
        conn, (provider.wallet as any).payer, mint,
        provider.wallet.publicKey, false, undefined, undefined,
        TOKEN_2022_PROGRAM_ID,
      );
    }

    for (let i = 0; i < 5; i++) {
      // Advance clock past cooldown
      await advanceClock(context, BigInt(61));

      const remainingAccounts = [];
      for (let k = 0; k < 6; k++) {
        const ata = getResourceAta(provider.wallet.publicKey, resourceMints[k]);
        remainingAccounts.push({ pubkey: resourceMints[k], isSigner: false, isWritable: true });
        remainingAccounts.push({ pubkey: ata, isSigner: false, isWritable: true });
      }

      await searchProgram.methods
        .runSearch()
        .accounts({
          owner: provider.wallet.publicKey,
          player: playerPdaAddr,
          gameConfig,
          searchAuthority: searchAuth,
          resourceAuthority: resAuth,
          recentSlothashes: SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
          resourceManagerProgram: PROGRAM_IDS.RESOURCE_MANAGER,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(remainingAccounts)
        .rpc();
    }

    const p = await searchProgram.account.player.fetch(playerPdaAddr);
    expect(p.searchNonce.toNumber()).to.equal(5);
  });

  // ── Phase 4: Craft all items ────────────────────────────────────────────────
  it("4. crafts all 4 items", async () => {
    const [gameConfig] = gameConfigPda();
    const [craftAuth] = craftingAuthorityPda();
    const [resAuth] = resourceAuthorityPda();
    const [colAuth] = collectionAuthorityPda();
    const [nftCfg] = itemNftConfigPda();
    const conn = provider.connection;

    for (let itemType = 0; itemType < 4; itemType++) {
      const assetKp = Keypair.generate();
      const [metadataPda] = itemMetadataPda(assetKp.publicKey);

      const remainingAccounts = [];
      for (let k = 0; k < 6; k++) {
        const ata = getResourceAta(provider.wallet.publicKey, resourceMints[k]);
        remainingAccounts.push({ pubkey: resourceMints[k], isSigner: false, isWritable: true });
        remainingAccounts.push({ pubkey: ata, isSigner: false, isWritable: true });
      }
      // NFT accounts
      remainingAccounts.push({ pubkey: assetKp.publicKey, isSigner: true, isWritable: true });
      remainingAccounts.push({ pubkey: provider.wallet.publicKey, isSigner: false, isWritable: true });
      remainingAccounts.push({ pubkey: metadataPda, isSigner: false, isWritable: true });
      remainingAccounts.push({ pubkey: nftCfg, isSigner: false, isWritable: false });
      remainingAccounts.push({ pubkey: collectionKeypair.publicKey, isSigner: false, isWritable: true });
      remainingAccounts.push({ pubkey: colAuth, isSigner: false, isWritable: false });

      await craftingProgram.methods
        .craftItem(itemType)
        .accounts({
          player: provider.wallet.publicKey,
          craftingAuthority: craftAuth,
          resourceAuthority: resAuth,
          gameConfig,
          resourceManagerProgram: PROGRAM_IDS.RESOURCE_MANAGER,
          itemNftProgram: PROGRAM_IDS.ITEM_NFT,
          mplCoreProgram: MPL_CORE_ID,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(remainingAccounts)
        .signers([assetKp])
        .rpc();

      // Verify ItemMetadata
      const meta = await itemNftProgram.account.itemMetadata.fetch(metadataPda);
      expect(meta.itemType).to.equal(itemType);
      expect(meta.owner.toBase58()).to.equal(provider.wallet.publicKey.toBase58());
    }
  });

  // ── Phase 5: Sell all items ─────────────────────────────────────────────────
  it("5. sells all 4 items and receives MagicToken", async () => {
    const [gameConfig] = gameConfigPda();
    const [mpAuth] = marketplaceAuthorityPda();
    const [colAuth] = collectionAuthorityPda();
    const [nftCfg] = itemNftConfigPda();
    const [magicAuth] = magicAuthorityPda();
    const [magicMint] = magicMintPda();
    const conn = provider.connection;

    // Create seller's MagicToken ATA
    await getOrCreateAssociatedTokenAccount(
      conn, (provider.wallet as any).payer, magicMint,
      provider.wallet.publicKey, false, undefined, undefined,
      TOKEN_2022_PROGRAM_ID,
    );
    const sellerMagicAta = getMagicAta(provider.wallet.publicKey, magicMint);

    const EXPECTED_TOTAL = 100 + 200 + 300 + 400; // sum of item_prices

    // Fetch the 4 crafted item metadata PDAs created in phase 4
    // (We would need to track asset keys from phase 4 — in a real test suite we'd
    //  use a shared fixture. Here we fetch all ItemMetadata accounts owned by the player.)
    const allMetadata = await itemNftProgram.account.itemMetadata.all([
      {
        memcmp: {
          offset: 8 + 1, // after discriminator + item_type
          bytes: provider.wallet.publicKey.toBase58(),
        },
      },
    ]);

    for (const { account, publicKey: metaPda } of allMetadata) {
      const assetPk = account.mint as PublicKey;
      const [expectedMeta] = itemMetadataPda(assetPk);
      expect(expectedMeta.toBase58()).to.equal(metaPda.toBase58());

      const itemType: number = account.itemType;

      const remaining = [
        { pubkey: collectionKeypair.publicKey, isSigner: false, isWritable: true },
        { pubkey: colAuth, isSigner: false, isWritable: false },
        { pubkey: nftCfg, isSigner: false, isWritable: false },
        { pubkey: magicAuth, isSigner: false, isWritable: false },
        { pubkey: magicMint, isSigner: false, isWritable: true },
        { pubkey: sellerMagicAta, isSigner: false, isWritable: true },
      ];

      await marketplaceProgram.methods
        .sellItem(itemType)
        .accounts({
          seller: provider.wallet.publicKey,
          marketplaceAuthority: mpAuth,
          asset: assetPk,
          itemMetadata: metaPda,
          gameConfig,
          itemNftProgram: PROGRAM_IDS.ITEM_NFT,
          magicTokenProgram: PROGRAM_IDS.MAGIC_TOKEN,
          mplCoreProgram: MPL_CORE_ID,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(remaining)
        .rpc();
    }

    // Verify MagicToken balance
    const magicAcct = await getAccount(conn, sellerMagicAta, undefined, TOKEN_2022_PROGRAM_ID);
    expect(Number(magicAcct.amount)).to.equal(EXPECTED_TOTAL);
  });
});
