/**
 * Скрипт розгортання "Козацький бізнес" на Solana Devnet.
 *
 * Послідовність ініціалізації:
 * 1. search::initialize_search_authority
 * 2. crafting::initialize_crafting_authority
 * 3. marketplace::initialize_marketplace_authority
 * 4. resource_manager::initialize_game(search_auth, crafting_auth, item_prices)
 * 5. resource_manager::create_resource_mint × 6 (WOOD, IRON, GOLD, LEATHER, STONE, DIAMOND)
 * 6. magic_token::initialize(marketplace_auth)
 * 7. item_nft::initialize_config(crafting_auth, marketplace_auth)
 *
 * Запуск: npx ts-node scripts/deploy.ts
 */

import * as anchor from "@anchor-lang/core";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";

const { BN, web3 } = anchor;

// Program IDs (з Anchor.toml / declare_id!)
const RESOURCE_MANAGER_ID = new PublicKey("G2FcuoLPQ8kSTTVLmKT9HR7b23em154skEidRVwTRtc9");
const MAGIC_TOKEN_ID      = new PublicKey("8KkKfeEqviwfGGnz1jLF5DVvhjC8HuSoXxnWJk4MQeoj");
const ITEM_NFT_ID         = new PublicKey("7cSkPT8JwQHibWDnXCqsADcM5euk5RRSffRsz6U5QCkR");
const SEARCH_ID           = new PublicKey("8cNt5T19ynDwaV8MkTBodqWnY4HydWZu4SdKDWL6ZU5X");
const CRAFTING_ID         = new PublicKey("9gbQXMjCev1K5GmK4SzBa52zKtXPveZQvVmaA15rvszs");
const MARKETPLACE_ID      = new PublicKey("9tTHLWqsKEiVufU7uqeAgSJmJrvXTsCkSvTKgdrb2ofX");

// Назви ресурсів (resource_id відповідає game_config.resource_mints[id])
const RESOURCE_NAMES = [
  { id: 0, name: "Дерево",   symbol: "WOOD",    uri: "https://kozatsky-biznes.game/resources/0" },
  { id: 1, name: "Залізо",   symbol: "IRON",    uri: "https://kozatsky-biznes.game/resources/1" },
  { id: 2, name: "Золото",   symbol: "GOLD",    uri: "https://kozatsky-biznes.game/resources/2" },
  { id: 3, name: "Шкіра",    symbol: "LEATHER", uri: "https://kozatsky-biznes.game/resources/3" },
  { id: 4, name: "Камінь",   symbol: "STONE",   uri: "https://kozatsky-biznes.game/resources/4" },
  { id: 5, name: "Діамант",  symbol: "DIAMOND", uri: "https://kozatsky-biznes.game/resources/5" },
];

// Ціни предметів у MagicToken
const ITEM_PRICES = [10, 20, 30, 50];

function loadIdl(name: string) {
  const idlPath = path.join(__dirname, "..", "target", "idl", `${name}.json`);
  return JSON.parse(fs.readFileSync(idlPath, "utf-8"));
}

function findPda(programId: PublicKey, seeds: Buffer[]) {
  return PublicKey.findProgramAddressSync(seeds, programId);
}

async function main() {
  // Налаштування провайдера
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = (provider.wallet as anchor.Wallet).payer;

  console.log(`Адміністратор: ${payer.publicKey.toBase58()}`);
  const balance = await provider.connection.getBalance(payer.publicKey);
  console.log(`Баланс: ${balance / web3.LAMPORTS_PER_SOL} SOL`);

  if (balance < 1e9) {
    console.error("Недостатньо SOL! Поповніть баланс через https://faucet.solana.com");
    process.exit(1);
  }

  // Ініціалізація програм
  const rmProgram  = new anchor.Program(loadIdl("resource_manager"), provider);
  const mtProgram  = new anchor.Program(loadIdl("magic_token"), provider);
  const nftProgram = new anchor.Program(loadIdl("item_nft"), provider);
  const srProgram  = new anchor.Program(loadIdl("search"), provider);
  const crProgram  = new anchor.Program(loadIdl("crafting"), provider);
  const mpProgram  = new anchor.Program(loadIdl("marketplace"), provider);

  // --- PDA адреси ---
  const [searchAuthPda]      = findPda(SEARCH_ID,      [Buffer.from("search_authority")]);
  const [craftingAuthPda]    = findPda(CRAFTING_ID,    [Buffer.from("crafting_authority")]);
  const [marketplaceAuthPda] = findPda(MARKETPLACE_ID, [Buffer.from("marketplace_authority")]);
  const [gameConfigPda]      = findPda(RESOURCE_MANAGER_ID, [Buffer.from("game_config")]);
  const [mintAuthorityPda]   = findPda(RESOURCE_MANAGER_ID, [Buffer.from("mint_authority")]);
  const [mtConfigPda]        = findPda(MAGIC_TOKEN_ID,      [Buffer.from("magic_token_config")]);
  const [mtMintAuthPda]      = findPda(MAGIC_TOKEN_ID,      [Buffer.from("mt_mint_authority")]);
  const [nftConfigPda]       = findPda(ITEM_NFT_ID,         [Buffer.from("item_nft_config")]);
  const [nftMintAuthPda]     = findPda(ITEM_NFT_ID,         [Buffer.from("nft_mint_authority")]);

  // --- STEP 1: search::initialize_search_authority ---
  console.log("\n[1/7] Ініціалізація SearchAuthority...");
  try {
    await srProgram.methods
      .initializeSearchAuthority()
      .accountsPartial({
        searchAuthority: searchAuthPda,
        admin: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log(`SearchAuthority: ${searchAuthPda.toBase58()}`);
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("SearchAuthority вже ініціалізовано");
    } else throw e;
  }

  // --- STEP 2: crafting::initialize_crafting_authority ---
  console.log("[2/7] Ініціалізація CraftingAuthority...");
  try {
    await crProgram.methods
      .initializeCraftingAuthority()
      .accountsPartial({
        craftingAuthority: craftingAuthPda,
        admin: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log(`CraftingAuthority: ${craftingAuthPda.toBase58()}`);
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("CraftingAuthority вже ініціалізовано");
    } else throw e;
  }

  // --- STEP 3: marketplace::initialize_marketplace_authority ---
  console.log("[3/7] Ініціалізація MarketplaceAuthority...");
  try {
    await mpProgram.methods
      .initializeMarketplaceAuthority()
      .accountsPartial({
        marketplaceAuthority: marketplaceAuthPda,
        admin: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log(`MarketplaceAuthority: ${marketplaceAuthPda.toBase58()}`);
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("MarketplaceAuthority вже ініціалізовано");
    } else throw e;
  }

  // --- STEP 4: resource_manager::initialize_game ---
  console.log("[4/7] Ініціалізація GameConfig...");
  try {
    await rmProgram.methods
      .initializeGame(
        searchAuthPda,
        craftingAuthPda,
        ITEM_PRICES.map(p => new BN(p))
      )
      .accountsPartial({
        gameConfig: gameConfigPda,
        mintAuthority: mintAuthorityPda,
        admin: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log(`GameConfig: ${gameConfigPda.toBase58()}`);
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("GameConfig вже ініціалізовано");
    } else throw e;
  }

  // --- STEP 5: resource_manager::create_resource_mint × 6 ---
  console.log("[5/7] Створення 6 ресурсних мінтів...");
  for (const resource of RESOURCE_NAMES) {
    const mintKeypair = Keypair.generate();
    console.log(`  Ресурс ${resource.id} (${resource.symbol}): ${mintKeypair.publicKey.toBase58()}`);
    try {
      await rmProgram.methods
        .createResourceMint(resource.id, resource.name, resource.symbol, resource.uri)
        .accountsPartial({
          gameConfig: gameConfigPda,
          mintAuthority: mintAuthorityPda,
          admin: payer.publicKey,
          mint: mintKeypair.publicKey,
          token2022Program: new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([mintKeypair])
        .rpc();
    } catch (e: any) {
      if (e.toString().includes("MintAlreadySet")) {
        console.log(`  Мінт ${resource.symbol} вже встановлено`);
      } else throw e;
    }
  }

  // --- STEP 6: magic_token::initialize ---
  console.log("[6/7] Ініціалізація MagicToken...");
  const magicMintKeypair = Keypair.generate();
  console.log(`  MagicToken мінт: ${magicMintKeypair.publicKey.toBase58()}`);
  try {
    await mtProgram.methods
      .initialize(marketplaceAuthPda)
      .accountsPartial({
        magicTokenConfig: mtConfigPda,
        mintAuthority: mtMintAuthPda,
        admin: payer.publicKey,
        mint: magicMintKeypair.publicKey,
        token2022Program: new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([magicMintKeypair])
      .rpc();
    console.log(`MagicTokenConfig: ${mtConfigPda.toBase58()}`);
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("MagicTokenConfig вже ініціалізовано");
    } else throw e;
  }

  // --- STEP 7: item_nft::initialize_config ---
  console.log("[7/7] Ініціалізація ItemNftConfig...");
  try {
    await nftProgram.methods
      .initializeConfig(craftingAuthPda, marketplaceAuthPda)
      .accountsPartial({
        itemNftConfig: nftConfigPda,
        nftMintAuthority: nftMintAuthPda,
        admin: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log(`ItemNftConfig: ${nftConfigPda.toBase58()}`);
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("ItemNftConfig вже ініціалізовано");
    } else throw e;
  }

  console.log("\n✓ Розгортання завершено!");
}

main().catch(console.error);
