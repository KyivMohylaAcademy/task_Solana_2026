/**
 * init.ts — One-time devnet initialisation script for Cossack Business.
 *
 * Usage:
 *   ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
 *   ANCHOR_WALLET=./deploy.json \
 *   npx ts-node app/scripts/init.ts
 *
 * What it does:
 *   1. Creates 6 resource mints (Token-2022 with MetadataPointer + TokenMetadata extensions).
 *   2. Creates the MagicToken mint (Token-2022).
 *   3. Initialises GameConfig with all mint addresses and item prices.
 *   4. Saves all generated addresses to devnet-accounts.json for use by play.ts.
 *
 * Run once per deployment. Re-running will fail because GameConfig already exists.
 */

import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import fs from "fs";
import { ResourceManager } from "../../target/types/resource_manager";
import { MagicToken } from "../../target/types/magic_token";

const DEVNET_ACCOUNTS_FILE = "devnet-accounts.json";

const RESOURCE_NAMES    = ["Wood", "Iron", "Gold", "Leather", "Stone", "Diamond"];
const RESOURCE_SYMBOLS  = ["WOOD", "IRON", "GOLD", "LTHR", "STON", "DIAM"];
// Replace REPLACE_ME with real hosted metadata JSON URLs before running on devnet.
const RESOURCE_URIS     = Array.from({ length: 6 }, (_, i) => `https://REPLACE_ME/resource${i}.json`);

const ITEM_PRICES = [
  new anchor.BN(10), // Cossack Saber
  new anchor.BN(15), // Elder's Staff
  new anchor.BN(20), // Kharakternyk's Armor
  new anchor.BN(25), // Battle Bracelet
];

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const rmProgram  = anchor.workspace.ResourceManager as anchor.Program<ResourceManager>;
  const mtProgram  = anchor.workspace.MagicToken     as anchor.Program<MagicToken>;

  const [resourceMintAuth] = PublicKey.findProgramAddressSync(
    [Buffer.from("resource_mint_auth")],
    rmProgram.programId,
  );
  const [magicMintAuth] = PublicKey.findProgramAddressSync(
    [Buffer.from("magic_mint_auth")],
    mtProgram.programId,
  );
  const [gameConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("game_config")],
    rmProgram.programId,
  );

  // ── 1. Create 6 resource mints ───────────────────────────────────────────────
  const resourceMintKeypairs: Keypair[] = [];
  for (let i = 0; i < 6; i++) {
    const mintKp = Keypair.generate();
    resourceMintKeypairs.push(mintKp);
    console.log(`Creating resource mint ${i} (${RESOURCE_NAMES[i]})…`);
    await rmProgram.methods
      .initResourceMint(i, RESOURCE_NAMES[i], RESOURCE_SYMBOLS[i], RESOURCE_URIS[i])
      .accounts({
        payer:           provider.wallet.publicKey,
        mint:            mintKp.publicKey,
        resourceMintAuth,
        tokenProgram:    TOKEN_2022_PROGRAM_ID,
        systemProgram:   SystemProgram.programId,
      })
      .signers([mintKp])
      .rpc();
    console.log(`  ✔ ${RESOURCE_NAMES[i]}: ${mintKp.publicKey.toBase58()}`);
  }

  // ── 2. Create MagicToken mint ────────────────────────────────────────────────
  const magicTokenMintKp = Keypair.generate();
  console.log("Creating MagicToken mint…");
  await mtProgram.methods
    .initMagicTokenMint("MagicToken", "MGT", "https://REPLACE_ME/magic-token.json")
    .accounts({
      payer:        provider.wallet.publicKey,
      mint:         magicTokenMintKp.publicKey,
      magicMintAuth,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([magicTokenMintKp])
    .rpc();
  console.log(`  ✔ MagicToken: ${magicTokenMintKp.publicKey.toBase58()}`);

  // ── 3. Initialise GameConfig ─────────────────────────────────────────────────
  console.log("Initialising GameConfig…");
  await rmProgram.methods
    .initialize(
      resourceMintKeypairs.map(kp => kp.publicKey),
      magicTokenMintKp.publicKey,
      ITEM_PRICES,
    )
    .accounts({
      admin:        provider.wallet.publicKey,
      gameConfig:   gameConfigPda,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  console.log(`  ✔ GameConfig: ${gameConfigPda.toBase58()}`);

  // ── 4. Save addresses ────────────────────────────────────────────────────────
  const addresses: Record<string, string> = {
    gameConfig:     gameConfigPda.toBase58(),
    magicTokenMint: magicTokenMintKp.publicKey.toBase58(),
  };
  for (let i = 0; i < 6; i++) {
    addresses[`resourceMint${i}`] = resourceMintKeypairs[i].publicKey.toBase58();
  }
  fs.writeFileSync(DEVNET_ACCOUNTS_FILE, JSON.stringify(addresses, null, 2));
  console.log(`\nAll addresses saved to ${DEVNET_ACCOUNTS_FILE}`);
}

main().catch(err => { console.error(err); process.exit(1); });
