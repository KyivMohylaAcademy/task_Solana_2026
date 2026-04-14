/**
 * End-to-end demo script: search → craft → sell on devnet.
 *
 * Prerequisites: run initialize.ts and register_player.ts first.
 *
 * Usage:
 *   pnpm exec ts-node scripts/demo.ts [--url devnet|localnet]
 */
import * as anchor from "@coral-xyz/anchor";
import {
  Connection, Keypair, PublicKey, SystemProgram, SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
  clusterApiUrl,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  getAccount,
} from "@solana/spl-token";
import fs from "fs";
import path from "path";
import {
  gameConfigPda, playerPda, searchAuthorityPda, resourceAuthorityPda,
  resourceMintPda, craftingAuthorityPda, marketplaceAuthorityPda,
  magicMintPda, magicAuthorityPda, collectionAuthorityPda, itemNftConfigPda,
  itemMetadataPda, getResourceAta, getMagicAta, PROGRAM_IDS,
} from "../tests/helpers/setup";

const MPL_CORE_ID = new PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJe1bpb");

async function sleep(ms: number) { return new Promise(r => setTimeout(r, ms)); }

async function main() {
  const args = process.argv.slice(2);
  const urlFlag = args.indexOf("--url");
  const cluster = urlFlag >= 0 ? args[urlFlag + 1] : "localnet";
  const rpcUrl = cluster === "devnet" ? clusterApiUrl("devnet") : "http://127.0.0.1:8899";

  const walletPath = process.env.ANCHOR_WALLET ?? `${process.env.HOME}/.config/solana/id.json`;
  const kpRaw = JSON.parse(fs.readFileSync(walletPath, "utf-8"));
  const owner = Keypair.fromSecretKey(Uint8Array.from(kpRaw));

  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = new anchor.Wallet(owner);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);

  const searchProgram = new anchor.Program(
    JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/search.json"), "utf-8")), provider);
  const craftingProgram = new anchor.Program(
    JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/crafting.json"), "utf-8")), provider);
  const marketplaceProgram = new anchor.Program(
    JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/marketplace.json"), "utf-8")), provider);
  const itemNftProgram = new anchor.Program(
    JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/item_nft.json"), "utf-8")), provider);

  const [gameConfig] = gameConfigPda();
  const [playerPdaAddr] = playerPda(owner.publicKey);
  const [searchAuth] = searchAuthorityPda();
  const [resAuth] = resourceAuthorityPda();
  const [craftAuth] = craftingAuthorityPda();
  const [mpAuth] = marketplaceAuthorityPda();
  const [magicMint] = magicMintPda();
  const [magicAuth] = magicAuthorityPda();
  const [colAuth] = collectionAuthorityPda();
  const [nftCfg] = itemNftConfigPda();

  const nftCfgData = await (itemNftProgram.account as any).itemNftConfig.fetch(nftCfg);
  const collectionPk: PublicKey = nftCfgData.collection;

  // Ensure resource ATAs exist
  const resourceMints: PublicKey[] = [];
  for (let k = 0; k < 6; k++) {
    const [mint] = resourceMintPda(k);
    resourceMints.push(mint);
    await getOrCreateAssociatedTokenAccount(
      connection, owner, mint, owner.publicKey, false, undefined, undefined, TOKEN_2022_PROGRAM_ID);
  }

  // ── Search ────────────────────────────────────────────────────────────────
  console.log("\n🔍 Running 3 searches (60s cooldown between each)...");
  for (let i = 0; i < 3; i++) {
    if (i > 0) {
      console.log("  Waiting 62 seconds for cooldown...");
      await sleep(62_000);
    }
    const remaining = resourceMints.flatMap(mint => [
      { pubkey: mint, isSigner: false, isWritable: true },
      { pubkey: getResourceAta(owner.publicKey, mint), isSigner: false, isWritable: true },
    ]);

    await (searchProgram.methods as any)
      .runSearch()
      .accounts({
        owner: owner.publicKey,
        player: playerPdaAddr,
        gameConfig,
        searchAuthority: searchAuth,
        resourceAuthority: resAuth,
        recentSlothashes: SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
        resourceManagerProgram: PROGRAM_IDS.RESOURCE_MANAGER,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts(remaining)
      .rpc();
    console.log(`  ✓ Search ${i + 1} complete`);
  }

  // ── Print resource balances ────────────────────────────────────────────────
  const NAMES = ["WOOD","IRON","GOLD","LEATHER","STONE","DIAMOND"];
  console.log("\nResource balances:");
  for (let k = 0; k < 6; k++) {
    const ata = getResourceAta(owner.publicKey, resourceMints[k]);
    const acct = await getAccount(connection, ata, undefined, TOKEN_2022_PROGRAM_ID);
    console.log(`  ${NAMES[k]}: ${acct.amount}`);
  }

  // ── Craft Saber (if resources allow) ──────────────────────────────────────
  console.log("\n⚒  Crafting Cossack Saber (item type 0)...");
  const assetKp = Keypair.generate();
  const [metaPda] = itemMetadataPda(assetKp.publicKey);

  const craftRemaining = [
    ...resourceMints.flatMap(mint => [
      { pubkey: mint, isSigner: false, isWritable: true },
      { pubkey: getResourceAta(owner.publicKey, mint), isSigner: false, isWritable: true },
    ]),
    { pubkey: assetKp.publicKey, isSigner: true, isWritable: true },
    { pubkey: owner.publicKey, isSigner: false, isWritable: true },
    { pubkey: metaPda, isSigner: false, isWritable: true },
    { pubkey: nftCfg, isSigner: false, isWritable: false },
    { pubkey: collectionPk, isSigner: false, isWritable: true },
    { pubkey: colAuth, isSigner: false, isWritable: false },
  ];

  await (craftingProgram.methods as any)
    .craftItem(0)
    .accounts({
      player: owner.publicKey,
      craftingAuthority: craftAuth,
      resourceAuthority: resAuth,
      gameConfig,
      resourceManagerProgram: PROGRAM_IDS.RESOURCE_MANAGER,
      itemNftProgram: PROGRAM_IDS.ITEM_NFT,
      mplCoreProgram: MPL_CORE_ID,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .remainingAccounts(craftRemaining)
    .signers([assetKp])
    .rpc();
  console.log(`  ✓ Saber minted: ${assetKp.publicKey.toBase58()}`);

  // ── Sell Saber ─────────────────────────────────────────────────────────────
  console.log("\n💰 Selling Saber...");
  await getOrCreateAssociatedTokenAccount(
    connection, owner, magicMint, owner.publicKey, false, undefined, undefined, TOKEN_2022_PROGRAM_ID);
  const sellerAta = getMagicAta(owner.publicKey, magicMint);

  const sellRemaining = [
    { pubkey: collectionPk, isSigner: false, isWritable: true },
    { pubkey: colAuth, isSigner: false, isWritable: false },
    { pubkey: nftCfg, isSigner: false, isWritable: false },
    { pubkey: magicAuth, isSigner: false, isWritable: false },
    { pubkey: magicMint, isSigner: false, isWritable: true },
    { pubkey: sellerAta, isSigner: false, isWritable: true },
  ];

  await (marketplaceProgram.methods as any)
    .sellItem(0)
    .accounts({
      seller: owner.publicKey,
      marketplaceAuthority: mpAuth,
      asset: assetKp.publicKey,
      itemMetadata: metaPda,
      gameConfig,
      itemNftProgram: PROGRAM_IDS.ITEM_NFT,
      magicTokenProgram: PROGRAM_IDS.MAGIC_TOKEN,
      mplCoreProgram: MPL_CORE_ID,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .remainingAccounts(sellRemaining)
    .rpc();

  const magicAcct = await getAccount(connection, sellerAta, undefined, TOKEN_2022_PROGRAM_ID);
  console.log(`  ✓ MagicToken balance: ${magicAcct.amount}`);
  console.log("\nDemo complete!");
}

main().catch(err => { console.error(err); process.exit(1); });
