/**
 * play.ts — End-to-end game-flow demo for Cossack Business on devnet.
 *
 * Usage:
 *   ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
 *   ANCHOR_WALLET=./deploy.json \
 *   npx ts-node app/scripts/play.ts
 *
 * Prerequisites: init.ts must have been run first (devnet-accounts.json must exist).
 *
 * What it does:
 *   1. Registers a Player PDA for the deploy wallet (idempotent — skips if already registered).
 *   2. Searches for resources once (succeeds, prints received resources).
 *   3. Immediately tries to search again (logs the CooldownNotElapsed error deliberately).
 *   4. Waits 61 seconds for the cooldown to expire.
 *   5. Searches again (succeeds).
 *   6. Admin-mints the Cossack Saber recipe (1×Wood + 3×Iron + 1×Leather) to ensure materials.
 *   7. Crafts a Cossack Saber NFT.
 *   8. Sells the Saber on the Marketplace, receives 10 MagicToken.
 */

import * as anchor from "@coral-xyz/anchor";
import {
  ComputeBudgetProgram,
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import fs from "fs";
import { ResourceManager } from "../../target/types/resource_manager";
import { MagicToken }      from "../../target/types/magic_token";
import { Search }          from "../../target/types/search";
import { ItemNft }         from "../../target/types/item_nft";
import { Crafting }        from "../../target/types/crafting";
import { Marketplace }     from "../../target/types/marketplace";

const DEVNET_ACCOUNTS_FILE = "devnet-accounts.json";
const MPL_TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
const SEARCH_COOLDOWN_MS = 61_000;

function sleep(ms: number) { return new Promise(r => setTimeout(r, ms)); }

function deriveMetadataPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    MPL_TOKEN_METADATA_PROGRAM_ID,
  )[0];
}

function deriveMasterEditionPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer(), Buffer.from("edition")],
    MPL_TOKEN_METADATA_PROGRAM_ID,
  )[0];
}

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const rmProgram          = anchor.workspace.ResourceManager as anchor.Program<ResourceManager>;
  const mtProgram          = anchor.workspace.MagicToken      as anchor.Program<MagicToken>;
  const searchProgram      = anchor.workspace.Search          as anchor.Program<Search>;
  const itemNftProgram     = anchor.workspace.ItemNft         as anchor.Program<ItemNft>;
  const craftingProgram    = anchor.workspace.Crafting        as anchor.Program<Crafting>;
  const marketplaceProgram = anchor.workspace.Marketplace     as anchor.Program<Marketplace>;

  // ── Load devnet addresses ────────────────────────────────────────────────────
  if (!fs.existsSync(DEVNET_ACCOUNTS_FILE)) {
    throw new Error(`${DEVNET_ACCOUNTS_FILE} not found — run init.ts first`);
  }
  const saved = JSON.parse(fs.readFileSync(DEVNET_ACCOUNTS_FILE, "utf-8"));
  const gameConfigPda   = new PublicKey(saved["gameConfig"]);
  const magicTokenMint  = new PublicKey(saved["magicTokenMint"]);
  const resourceMints   = Array.from({ length: 6 }, (_, i) => new PublicKey(saved[`resourceMint${i}`]));

  // ── Derive PDAs ──────────────────────────────────────────────────────────────
  const [resourceMintAuth] = PublicKey.findProgramAddressSync(
    [Buffer.from("resource_mint_auth")], rmProgram.programId,
  );
  const [magicMintAuth] = PublicKey.findProgramAddressSync(
    [Buffer.from("magic_mint_auth")], mtProgram.programId,
  );
  const [playerPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("player"), provider.wallet.publicKey.toBuffer()], searchProgram.programId,
  );
  const [nftAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("nft_authority")], itemNftProgram.programId,
  );
  const [searchCpiAuth] = PublicKey.findProgramAddressSync(
    [Buffer.from("cpi_auth")], searchProgram.programId,
  );
  const [craftingCpiAuth] = PublicKey.findProgramAddressSync(
    [Buffer.from("cpi_auth")], craftingProgram.programId,
  );
  const [marketplaceCpiAuth] = PublicKey.findProgramAddressSync(
    [Buffer.from("cpi_auth")], marketplaceProgram.programId,
  );

  const playerAtas = resourceMints.map(m =>
    getAssociatedTokenAddressSync(m, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID)
  );

  const searchAccounts = {
    playerWallet:          provider.wallet.publicKey,
    player:                playerPda,
    owner:                 provider.wallet.publicKey,
    gameConfig:            gameConfigPda,
    mint0: resourceMints[0], mint1: resourceMints[1], mint2: resourceMints[2],
    mint3: resourceMints[3], mint4: resourceMints[4], mint5: resourceMints[5],
    ata0: playerAtas[0], ata1: playerAtas[1], ata2: playerAtas[2],
    ata3: playerAtas[3], ata4: playerAtas[4], ata5: playerAtas[5],
    cpiAuth:               searchCpiAuth,
    resourceMintAuth,
    resourceManagerProgram: rmProgram.programId,
    tokenProgram:          TOKEN_2022_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram:         SystemProgram.programId,
  };

  // ── 1. Register player (idempotent) ──────────────────────────────────────────
  console.log("Registering player…");
  try {
    await searchProgram.methods.registerPlayer()
      .accounts({ signer: provider.wallet.publicKey, player: playerPda, systemProgram: SystemProgram.programId })
      .rpc();
    console.log("  ✔ Player registered:", playerPda.toBase58());
  } catch (e: any) {
    if (e.message?.includes("already in use")) {
      console.log("  (player already registered, skipping)");
    } else throw e;
  }

  // ── 2. Search once ───────────────────────────────────────────────────────────
  console.log("Searching for resources (attempt 1)…");
  await searchProgram.methods.searchResources()
    .accounts(searchAccounts)
    .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
    .rpc();
  console.log("  ✔ Search succeeded — 3 resources minted");

  // ── 3. Immediate second search (expected to fail with cooldown) ───────────────
  console.log("Searching immediately again (should fail with CooldownNotElapsed)…");
  try {
    await searchProgram.methods.searchResources()
      .accounts(searchAccounts)
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
      .rpc();
    console.log("  (unexpected success — cooldown may not be enforced)");
  } catch (e: any) {
    console.log("  ✔ Correctly rejected:", e.message?.match(/CooldownNotElapsed|custom program error/)?.[0] ?? e.message);
  }

  // ── 4. Wait for cooldown ─────────────────────────────────────────────────────
  console.log(`Waiting ${SEARCH_COOLDOWN_MS / 1000}s for cooldown…`);
  await sleep(SEARCH_COOLDOWN_MS);

  // ── 5. Search again ──────────────────────────────────────────────────────────
  console.log("Searching for resources (attempt 2, post-cooldown)…");
  await searchProgram.methods.searchResources()
    .accounts(searchAccounts)
    .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
    .rpc();
  console.log("  ✔ Search succeeded — 3 more resources minted");

  // ── 6. Admin-mint Cossack Saber recipe materials (1×Wood + 3×Iron + 1×Leather) ─
  console.log("Admin-minting recipe materials for Cossack Saber…");
  const saberAmounts = [1, 3, 0, 1, 0, 0]; // [Wood, Iron, Gold, Leather, Stone, Diamond]
  for (let i = 0; i < 6; i++) {
    if (saberAmounts[i] === 0) continue;
    await rmProgram.methods
      .adminMintResource(i, new anchor.BN(saberAmounts[i]))
      .accounts({
        admin:                 provider.wallet.publicKey,
        gameConfig:            gameConfigPda,
        mint:                  resourceMints[i],
        recipientAta:          playerAtas[i],
        recipient:             provider.wallet.publicKey,
        resourceMintAuth,
        tokenProgram:          TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram:         SystemProgram.programId,
      })
      .rpc();
  }
  console.log("  ✔ Materials minted");

  // ── 7. Craft Cossack Saber ───────────────────────────────────────────────────
  console.log("Crafting Cossack Saber (item_type=0)…");
  const nftMint = Keypair.generate();
  const recipientNftAta = getAssociatedTokenAddressSync(
    nftMint.publicKey, provider.wallet.publicKey, false, TOKEN_PROGRAM_ID,
  );
  const metadata      = deriveMetadataPda(nftMint.publicKey);
  const masterEdition = deriveMasterEditionPda(nftMint.publicKey);
  const [itemMetadataPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("item_metadata"), nftMint.publicKey.toBuffer()], itemNftProgram.programId,
  );

  await craftingProgram.methods.craftItem(0)
    .accounts({
      playerWallet:          provider.wallet.publicKey,
      nftMint:               nftMint.publicKey,
      gameConfig:            gameConfigPda,
      cpiAuth:               craftingCpiAuth,
      resourceMintAuth,
      mint0: resourceMints[0], mint1: resourceMints[1], mint2: resourceMints[2],
      mint3: resourceMints[3], mint4: resourceMints[4], mint5: resourceMints[5],
      ata0: playerAtas[0], ata1: playerAtas[1], ata2: playerAtas[2],
      ata3: playerAtas[3], ata4: playerAtas[4], ata5: playerAtas[5],
      nftAuthority,
      metadata,
      masterEdition,
      itemMetadata:          itemMetadataPda,
      recipientNftAta,
      resourceManagerProgram: rmProgram.programId,
      itemNftProgram:        itemNftProgram.programId,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      token2022Program:      TOKEN_2022_PROGRAM_ID,
      tokenProgram:          TOKEN_PROGRAM_ID,
      systemProgram:         SystemProgram.programId,
    })
    .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
    .signers([nftMint])
    .rpc();
  console.log("  ✔ NFT minted:", nftMint.publicKey.toBase58());

  // ── 8. Sell on Marketplace ───────────────────────────────────────────────────
  console.log("Selling Cossack Saber on marketplace…");
  const sellerMagicAta = getAssociatedTokenAddressSync(
    magicTokenMint, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID,
  );

  await marketplaceProgram.methods.sellItem()
    .accounts({
      seller:               provider.wallet.publicKey,
      gameConfig:           gameConfigPda,
      nftMint:              nftMint.publicKey,
      sellerNftAta:         recipientNftAta,
      itemMetadata:         itemMetadataPda,
      nftAuthority,
      cpiAuth:              marketplaceCpiAuth,
      magicTokenMint,
      sellerMagicAta,
      magicMintAuth,
      itemNftProgram:       itemNftProgram.programId,
      magicTokenProgram:    mtProgram.programId,
      tokenProgram:         TOKEN_PROGRAM_ID,
      token2022Program:     TOKEN_2022_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram:        SystemProgram.programId,
    })
    .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
    .rpc();

  const magicBalance = await provider.connection.getTokenAccountBalance(sellerMagicAta);
  console.log(`  ✔ NFT sold — received ${magicBalance.value.uiAmount} MagicToken`);
  console.log("\nDone.");
}

main().catch(err => { console.error(err); process.exit(1); });
