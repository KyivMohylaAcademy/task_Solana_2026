/**
 * Deployment initialization script.
 *
 * Runs two-phase setup on Devnet (or localnet):
 *   1. initialize_config (resource_manager)
 *   2. create_resource_mint × 6
 *   3. magic_token::initialize
 *   4. resource_manager::set_magic_token_mint
 *   5. item_nft::initialize_collection
 *
 * Usage:
 *   pnpm exec ts-node scripts/initialize.ts [--url devnet|localnet]
 */
import * as anchor from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, clusterApiUrl } from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import fs from "fs";
import path from "path";
import {
  gameConfigPda, resourceAuthorityPda, resourceMintPda,
  magicMintPda, magicAuthorityPda, magicConfigPda,
  collectionAuthorityPda, itemNftConfigPda, PROGRAM_IDS,
} from "../tests/helpers/setup";

const MPL_CORE_ID = new PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");

async function main() {
  const args = process.argv.slice(2);
  const urlFlag = args.indexOf("--url");
  const cluster = urlFlag >= 0 ? args[urlFlag + 1] : "localnet";
  const rpcUrl = cluster === "devnet" ? clusterApiUrl("devnet") : "http://127.0.0.1:8899";

  const walletPath = process.env.ANCHOR_WALLET ?? `${process.env.HOME}/.config/solana/id.json`;
  const keypairRaw = JSON.parse(fs.readFileSync(walletPath, "utf-8"));
  const admin = Keypair.fromSecretKey(Uint8Array.from(keypairRaw));

  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = new anchor.Wallet(admin);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);

  console.log(`\nAdmin: ${admin.publicKey.toBase58()}`);
  console.log(`Cluster: ${cluster} (${rpcUrl})\n`);

  // Load programs
  const rmIdl = JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/resource_manager.json"), "utf-8"));
  const mtIdl = JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/magic_token.json"), "utf-8"));
  const nftIdl = JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/item_nft.json"), "utf-8"));
  const rmProgram = new anchor.Program(rmIdl, provider);
  const mtProgram = new anchor.Program(mtIdl, provider);
  const nftProgram = new anchor.Program(nftIdl, provider);

  // ── 1. initialize_config ──────────────────────────────────────────────────
  const [configPda] = gameConfigPda();
  try {
    await (rmProgram.methods as any)
      .initializeConfig([
        new anchor.BN(100_000), // Saber: 100,000 MAGIC
        new anchor.BN(150_000), // Staff: 150,000 MAGIC
        new anchor.BN(200_000), // Armor: 200,000 MAGIC
        new anchor.BN(300_000), // Bracelet: 300,000 MAGIC
      ])
      .accounts({ admin: admin.publicKey, gameConfig: configPda, systemProgram: SystemProgram.programId })
      .rpc();
    console.log("✓ GameConfig initialized:", configPda.toBase58());
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  GameConfig already exists, skipping.");
    } else throw e;
  }

  // ── 2. create resource mints ──────────────────────────────────────────────
  const NAMES = ["WOOD","IRON","GOLD","LEATHER","STONE","DIAMOND"];
  const [resAuth] = resourceAuthorityPda();

  for (let kind = 0; kind < 6; kind++) {
    const [mintPda] = resourceMintPda(kind);
    try {
      await (rmProgram.methods as any)
        .createResourceMint(kind)
        .accounts({
          admin: admin.publicKey,
          gameConfig: configPda,
          mint: mintPda,
          resourceAuthority: resAuth,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .rpc();
      console.log(`✓ Resource mint [${NAMES[kind]}]: ${mintPda.toBase58()}`);
    } catch (e: any) {
      if (e.message?.includes("already in use") || e.message?.includes("MintAlreadyCreated")) {
        console.log(`  Mint [${NAMES[kind]}] already exists, skipping.`);
      } else throw e;
    }
  }

  // ── 3. initialize MagicToken ──────────────────────────────────────────────
  const [magicMint] = magicMintPda();
  const [magicAuth] = magicAuthorityPda();
  const [magicCfg] = magicConfigPda();
  try {
    await (mtProgram.methods as any)
      .initialize()
      .accounts({
        admin: admin.publicKey,
        magicMint,
        magicAuthority: magicAuth,
        magicConfig: magicCfg,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc();
    console.log("✓ MagicToken mint:", magicMint.toBase58());
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  MagicToken already initialized, skipping.");
    } else throw e;
  }

  // ── 4. link magic mint into GameConfig ────────────────────────────────────
  try {
    await (rmProgram.methods as any)
      .setMagicTokenMint(magicMint)
      .accounts({ admin: admin.publicKey, gameConfig: configPda })
      .rpc();
    console.log("✓ MagicToken registered in GameConfig");
  } catch (e: any) {
    if (e.message?.includes("MagicMintAlreadySet")) {
      console.log("  MagicToken already registered, skipping.");
    } else throw e;
  }

  // ── 5. initialize collection ──────────────────────────────────────────────
  const collectionKp = Keypair.generate();
  const [colAuth] = collectionAuthorityPda();
  const [nftCfg] = itemNftConfigPda();
  try {
    await (nftProgram.methods as any)
      .initializeCollection()
      .accounts({
        admin: admin.publicKey,
        collection: collectionKp.publicKey,
        collectionAuthority: colAuth,
        itemNftConfig: nftCfg,
        mplCoreProgram: MPL_CORE_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([collectionKp])
      .rpc();
    console.log("✓ NFT Collection:", collectionKp.publicKey.toBase58());
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  Collection already initialized, skipping.");
    } else throw e;
  }

  // ── Summary ───────────────────────────────────────────────────────────────
  console.log("\n═══════════════════════════════════════");
  console.log("Program IDs:");
  for (const [name, id] of Object.entries(PROGRAM_IDS)) {
    console.log(`  ${name}: ${id.toBase58()}`);
  }
  console.log("\nImportant PDAs:");
  console.log(`  GameConfig: ${configPda.toBase58()}`);
  console.log(`  MagicMint: ${magicMint.toBase58()}`);
  for (let k = 0; k < 6; k++) {
    const [m] = resourceMintPda(k);
    console.log(`  ${NAMES[k]} Mint: ${m.toBase58()}`);
  }
  console.log("═══════════════════════════════════════\n");
}

main().catch(err => { console.error(err); process.exit(1); });
