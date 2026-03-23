import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ResourceManager } from "../target/types/resource_manager";
import { MagicToken } from "../target/types/magic_token";
import { ItemNft } from "../target/types/item_nft";
import { Search } from "../target/types/search";
import { Crafting } from "../target/types/crafting";
import { Marketplace } from "../target/types/marketplace";
import {
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";

/**
 * Initialize all game state after programs are deployed.
 *
 * Usage:
 *   npx ts-node scripts/initialize.ts
 *
 * Environment:
 *   ANCHOR_PROVIDER_URL  – RPC endpoint (default: http://localhost:8899)
 *   ANCHOR_WALLET        – path to admin keypair (default: ~/.config/solana/id.json)
 */
async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const rmProgram = anchor.workspace.resourceManager as Program<ResourceManager>;
  const mtProgram = anchor.workspace.magicToken as Program<MagicToken>;
  const inProgram = anchor.workspace.itemNft as Program<ItemNft>;
  const searchProgram = anchor.workspace.search as Program<Search>;
  const craftProgram = anchor.workspace.crafting as Program<Crafting>;
  const marketProgram = anchor.workspace.marketplace as Program<Marketplace>;

  const admin = provider.wallet;
  console.log("Admin:", admin.publicKey.toBase58());

  const itemPrices = [100, 150, 200, 300];
  const rarityWeights = [30, 25, 20, 12, 10, 3];
  const searchCooldown = 60;

  const resourceNames = ["Wood", "Iron", "Gold", "Leather", "Stone", "Diamond"];
  const resourceSymbols = ["WOOD", "IRON", "GOLD", "LEATHER", "STONE", "DIAMOND"];

  // ── Derive PDAs ──────────────────────────────────────────────────────
  const [gameConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("game_config")],
    rmProgram.programId
  );
  const [mintAuthorityPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("mint_authority")],
    rmProgram.programId
  );
  const [magicConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("magic_config")],
    mtProgram.programId
  );
  const [magicMintAuthPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("magic_mint_authority")],
    mtProgram.programId
  );
  const [itemNftConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("item_nft_config")],
    inProgram.programId
  );
  const [nftAuthorityPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("nft_authority")],
    inProgram.programId
  );

  // ── 1. Initialize GameConfig ─────────────────────────────────────────
  console.log("\n1. Initializing GameConfig...");
  try {
    await rmProgram.methods
      .initializeGame(
        itemPrices.map((p) => new anchor.BN(p)),
        Buffer.from(rarityWeights),
        new anchor.BN(searchCooldown),
        searchProgram.programId,
        craftProgram.programId,
        marketProgram.programId
      )
      .accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPda,
        mintAuthority: mintAuthorityPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("   GameConfig PDA:", gameConfigPda.toBase58());
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("   GameConfig already initialized, skipping.");
    } else {
      throw e;
    }
  }

  // ── 2. Initialize 6 Resource Mints ───────────────────────────────────
  console.log("\n2. Initializing resource mints...");
  const resourceMints: Keypair[] = [];
  for (let i = 0; i < 6; i++) {
    const mintKp = Keypair.generate();
    resourceMints.push(mintKp);
    try {
      await rmProgram.methods
        .initializeResource(
          i,
          resourceNames[i],
          resourceSymbols[i],
          `https://cossack.game/resource/${i}.json`
        )
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          mint: mintKp.publicKey,
          mintAuthority: mintAuthorityPda,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([mintKp])
        .rpc();
      console.log(`   Resource ${i} (${resourceSymbols[i]}): ${mintKp.publicKey.toBase58()}`);
    } catch (e: any) {
      console.log(`   Resource ${i} error:`, e.message?.slice(0, 80));
    }
  }

  // ── 3. Initialize MagicToken ─────────────────────────────────────────
  console.log("\n3. Initializing MagicToken...");
  const magicMintKp = Keypair.generate();
  try {
    await mtProgram.methods
      .initializeMagicToken(
        "MagicToken",
        "MAGIC",
        "https://cossack.game/magic.json",
        marketProgram.programId
      )
      .accounts({
        admin: admin.publicKey,
        config: magicConfigPda,
        mint: magicMintKp.publicKey,
        mintAuthority: magicMintAuthPda,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([magicMintKp])
      .rpc();
    console.log("   MagicToken Mint:", magicMintKp.publicKey.toBase58());
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("   MagicToken already initialized, skipping.");
    } else {
      throw e;
    }
  }

  // ── 4. Initialize ItemNft Config ─────────────────────────────────────
  console.log("\n4. Initializing ItemNft config...");
  try {
    await inProgram.methods
      .initializeItemNft(craftProgram.programId, marketProgram.programId)
      .accounts({
        admin: admin.publicKey,
        config: itemNftConfigPda,
        nftAuthority: nftAuthorityPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("   ItemNftConfig PDA:", itemNftConfigPda.toBase58());
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("   ItemNft config already initialized, skipping.");
    } else {
      throw e;
    }
  }

  console.log("\n✅ All game state initialized successfully!");
  console.log("\nProgram IDs:");
  console.log("  resource_manager:", rmProgram.programId.toBase58());
  console.log("  magic_token:     ", mtProgram.programId.toBase58());
  console.log("  item_nft:        ", inProgram.programId.toBase58());
  console.log("  search:          ", searchProgram.programId.toBase58());
  console.log("  crafting:        ", craftProgram.programId.toBase58());
  console.log("  marketplace:     ", marketProgram.programId.toBase58());
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
