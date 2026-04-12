/**
 * init.ts — Initialises all programs for "Козацький бізнес".
 *
 * Run after deploying all programs:
 *   npx ts-node scripts/init.ts
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Keypair, Transaction } from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getMintLen,
  createInitializeMintInstruction,
} from "@solana/spl-token";
import { ResourceManager } from "../target/types/resource_manager";
import { MagicToken }      from "../target/types/magic_token";
import { ItemNft }         from "../target/types/item_nft";
import { Marketplace }     from "../target/types/marketplace";

const RESOURCE_NAMES   = ["Дерево","Залізо","Золото","Шкіра","Камінь","Алмаз"];
const RESOURCE_SYMBOLS = ["WOOD","IRON","GOLD","LEATHER","STONE","DIAMOND"];

async function createMint(
  connection: anchor.web3.Connection,
  payer: Keypair,
  mintAuthority: PublicKey,
  freezeAuthority: PublicKey | null,
  decimals: number,
): Promise<PublicKey> {
  const mintKp  = Keypair.generate();
  const mintLen = getMintLen([]);
  const lamports = await connection.getMinimumBalanceForRentExemption(mintLen);

  const tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey:       payer.publicKey,
      newAccountPubkey: mintKp.publicKey,
      space:            mintLen,
      lamports,
      programId:        TOKEN_2022_PROGRAM_ID,
    }),
    createInitializeMintInstruction(
      mintKp.publicKey, decimals, mintAuthority, freezeAuthority, TOKEN_2022_PROGRAM_ID,
    ),
  );
  await anchor.web3.sendAndConfirmTransaction(connection, tx, [payer, mintKp]);
  console.log(`  Created mint: ${mintKp.publicKey.toBase58()}`);
  return mintKp.publicKey;
}

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const admin = (provider.wallet as anchor.Wallet).payer;
  const conn  = provider.connection;

  const rmProgram  = anchor.workspace.ResourceManager as Program<ResourceManager>;
  const mtProgram  = anchor.workspace.MagicToken      as Program<MagicToken>;
  const nftProgram = anchor.workspace.ItemNft         as Program<ItemNft>;
  const mpProgram  = anchor.workspace.Marketplace     as Program<Marketplace>;

  // ── Derive PDAs ─────────────────────────────────────────────────────────────
  const [gameConfigPda]           = PublicKey.findProgramAddressSync([Buffer.from("game_config")],           rmProgram.programId);
  const [resourceMintAuthPda]     = PublicKey.findProgramAddressSync([Buffer.from("mint_authority")],        rmProgram.programId);
  const [magicTokenConfigPda]     = PublicKey.findProgramAddressSync([Buffer.from("magic_token_config")],    mtProgram.programId);
  const [magicMintAuthPda]        = PublicKey.findProgramAddressSync([Buffer.from("magic_mint_authority")],  mtProgram.programId);
  const [itemNftConfigPda]        = PublicKey.findProgramAddressSync([Buffer.from("item_nft_config")],       nftProgram.programId);
  const [itemNftAuthorityPda]     = PublicKey.findProgramAddressSync([Buffer.from("item_nft_authority")],    nftProgram.programId);
  const [searchAuthorityPda]      = PublicKey.findProgramAddressSync([Buffer.from("search_authority")],      anchor.workspace.Search.programId);
  const [craftingAuthorityPda]    = PublicKey.findProgramAddressSync([Buffer.from("crafting_authority")],    anchor.workspace.Crafting.programId);
  const [marketplaceConfigPda]    = PublicKey.findProgramAddressSync([Buffer.from("marketplace_config")],    mpProgram.programId);
  const [marketplaceAuthorityPda] = PublicKey.findProgramAddressSync([Buffer.from("marketplace_authority")], mpProgram.programId);

  console.log("\n=== Ініціалізація Козацький Бізнес ===\n");

  // ── 1. Initialize resource_manager ─────────────────────────────────────────
  console.log("1. Ініціалізація resource_manager...");
  await rmProgram.methods
    .initializeGame(
      searchAuthorityPda,
      craftingAuthorityPda,
      marketplaceAuthorityPda,
      [new anchor.BN(10), new anchor.BN(20), new anchor.BN(30), new anchor.BN(50)],
    )
    .accounts({ gameConfig: gameConfigPda, admin: admin.publicKey, systemProgram: SystemProgram.programId })
    .rpc();
  console.log(`   GameConfig: ${gameConfigPda.toBase58()}`);

  // ── 2. Create & register 6 resource mints ──────────────────────────────────
  console.log("\n2. Створення ресурсних мінтів (SPL Token-2022)...");
  const resourceMints: PublicKey[] = [];
  for (let i = 0; i < 6; i++) {
    const mint = await createMint(conn, admin, resourceMintAuthPda, resourceMintAuthPda, 0);
    resourceMints.push(mint);
    await rmProgram.methods.registerResourceMint(i)
      .accounts({ gameConfig: gameConfigPda, admin: admin.publicKey, resourceMint: mint })
      .rpc();
    console.log(`   ${RESOURCE_SYMBOLS[i]} (${i}): ${mint.toBase58()}`);
  }

  // ── 3. Create MagicToken mint ───────────────────────────────────────────────
  console.log("\n3. Створення MagicToken мінту...");
  const magicMint = await createMint(conn, admin, magicMintAuthPda, null, 0);

  await mtProgram.methods
    .initialize(marketplaceAuthorityPda)
    .accounts({
      magicTokenConfig: magicTokenConfigPda,
      magicMint,
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  console.log(`   MagicToken mint: ${magicMint.toBase58()}`);

  await rmProgram.methods.registerMagicTokenMint()
    .accounts({ gameConfig: gameConfigPda, admin: admin.publicKey, magicTokenMint: magicMint })
    .rpc();

  // ── 4. Initialize item_nft ─────────────────────────────────────────────────
  console.log("\n4. Ініціалізація item_nft...");
  await nftProgram.methods
    .initialize(craftingAuthorityPda, marketplaceAuthorityPda)
    .accounts({
      itemNftConfig: itemNftConfigPda,
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  console.log(`   ItemNftConfig: ${itemNftConfigPda.toBase58()}`);

  // ── 5. Initialize marketplace ──────────────────────────────────────────────
  console.log("\n5. Ініціалізація marketplace...");
  await mpProgram.methods
    .initialize([new anchor.BN(10), new anchor.BN(20), new anchor.BN(30), new anchor.BN(50)])
    .accounts({
      marketplaceConfig: marketplaceConfigPda,
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
  console.log(`   MarketplaceConfig: ${marketplaceConfigPda.toBase58()}`);

  console.log("\n=== ✅ Ініціалізація завершена! ===\n");
  console.log("Program IDs:");
  console.log(`  resource_manager: ${rmProgram.programId.toBase58()}`);
  console.log(`  magic_token:      ${mtProgram.programId.toBase58()}`);
  console.log(`  item_nft:         ${nftProgram.programId.toBase58()}`);
  console.log(`  marketplace:      ${mpProgram.programId.toBase58()}`);
  console.log("\nResource Mints:");
  resourceMints.forEach((m, i) => console.log(`  ${RESOURCE_SYMBOLS[i]}: ${m.toBase58()}`));
  console.log(`  MAGIC: ${magicMint.toBase58()}`);
}

main().catch(console.error);
