/**
 * End-to-end тест гри "Козацький бізнес".
 *
 * Сценарій: search → craft → sell
 * 1. Реєструємо гравця (search::initialize_player)
 * 2. Пошук ресурсів (search::search_resources)
 * 3. Крафт предмету (crafting::craft_item)
 * 4. Продаж предмету (marketplace::sell_item)
 *
 * Запуск: npx ts-node scripts/test-game.ts
 * Передумови: виконати deploy.ts першим.
 */

import * as anchor from "@anchor-lang/core";
import {
  PublicKey, Keypair, SystemProgram,
  SYSVAR_RENT_PUBKEY, LAMPORTS_PER_SOL
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import * as fs from "fs";
import * as path from "path";

const { BN, web3 } = anchor;

const RESOURCE_MANAGER_ID = new PublicKey("G2FcuoLPQ8kSTTVLmKT9HR7b23em154skEidRVwTRtc9");
const MAGIC_TOKEN_ID      = new PublicKey("8KkKfeEqviwfGGnz1jLF5DVvhjC8HuSoXxnWJk4MQeoj");
const ITEM_NFT_ID         = new PublicKey("7cSkPT8JwQHibWDnXCqsADcM5euk5RRSffRsz6U5QCkR");
const SEARCH_ID           = new PublicKey("8cNt5T19ynDwaV8MkTBodqWnY4HydWZu4SdKDWL6ZU5X");
const CRAFTING_ID         = new PublicKey("9gbQXMjCev1K5GmK4SzBa52zKtXPveZQvVmaA15rvszs");
const MARKETPLACE_ID      = new PublicKey("9tTHLWqsKEiVufU7uqeAgSJmJrvXTsCkSvTKgdrb2ofX");

// Рецепт Шаблі козака: IRON(1)×3 + WOOD(0)×1 + LEATHER(3)×1
const SABER_RECIPE = [
  { resourceId: 1, amount: 3 },  // IRON
  { resourceId: 0, amount: 1 },  // WOOD
  { resourceId: 3, amount: 1 },  // LEATHER
];

function loadIdl(name: string) {
  const idlPath = path.join(__dirname, "..", "target", "idl", `${name}.json`);
  return JSON.parse(fs.readFileSync(idlPath, "utf-8"));
}

function findPda(programId: PublicKey, seeds: Buffer[]) {
  return PublicKey.findProgramAddressSync(seeds, programId);
}

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const payer = (provider.wallet as anchor.Wallet).payer;

  console.log(`Гравець: ${payer.publicKey.toBase58()}`);

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
  const [playerPda]          = findPda(SEARCH_ID, [Buffer.from("player"), payer.publicKey.toBuffer()]);
  const [mtConfigPda]        = findPda(MAGIC_TOKEN_ID, [Buffer.from("magic_token_config")]);
  const [mtMintAuthPda]      = findPda(MAGIC_TOKEN_ID, [Buffer.from("mt_mint_authority")]);
  const [nftConfigPda]       = findPda(ITEM_NFT_ID, [Buffer.from("item_nft_config")]);
  const [nftMintAuthPda]     = findPda(ITEM_NFT_ID, [Buffer.from("nft_mint_authority")]);

  // Читаємо стан гри
  const gameConfig = await rmProgram.account.gameConfig.fetch(gameConfigPda);
  const mtConfig   = await mtProgram.account.magicTokenConfig.fetch(mtConfigPda);

  console.log(`\nМінти ресурсів:`);
  gameConfig.resourceMints.forEach((mint: PublicKey, i: number) => {
    console.log(`  [${i}]: ${mint.toBase58()}`);
  });

  // --- STEP 1: Реєстрація гравця ---
  console.log("\n[1] Реєстрація гравця...");
  try {
    await srProgram.methods
      .initializePlayer()
      .accountsPartial({
        player: playerPda,
        playerWallet: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log(`Player PDA: ${playerPda.toBase58()}`);
  } catch (e: any) {
    if (e.toString().includes("already in use")) {
      console.log("Гравець вже зареєстрований");
    } else throw e;
  }

  // --- STEP 2: Пошук ресурсів (потрібно кілька раз для крафту) ---
  console.log("\n[2] Пошук ресурсів (щонайменше 4 рази для крафту)...");
  const needMoreSearches = true;
  let searchCount = 0;
  while (searchCount < 4) {
    // Отримуємо стан гравця для псевдовипадкового розрахунку
    const clock = await provider.connection.getSlot();
    const slotInfo = await provider.connection.getSlot("confirmed");

    // Визначаємо resource_id (логіка з search_resources.rs)
    const playerAccount = await srProgram.account.player.fetch(playerPda).catch(() => ({ lastSearchTimestamp: new BN(0) }));
    const timestamp = Math.floor(Date.now() / 1000);
    const lastSearch = (playerAccount as any).lastSearchTimestamp?.toNumber() || 0;
    const seed = BigInt(slotInfo) + BigInt(timestamp) + BigInt(lastSearch);
    const resourceId = Number(seed % 6n);

    const resourceMint = gameConfig.resourceMints[resourceId];
    const playerTokenAccount = getAssociatedTokenAddressSync(
      resourceMint, payer.publicKey, false, TOKEN_2022_PROGRAM_ID
    );

    console.log(`  Пошук ${searchCount + 1}: ресурс ${resourceId}`);
    try {
      await srProgram.methods
        .searchResources()
        .accountsPartial({
          gameConfig: gameConfigPda,
          mintAuthority: mintAuthorityPda,
          searchAuthority: searchAuthPda,
          player: playerPda,
          playerWallet: payer.publicKey,
          resourceMint: resourceMint,
          playerTokenAccount: playerTokenAccount,
          resourceManagerProgram: RESOURCE_MANAGER_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
        })
        .rpc();
      searchCount++;
    } catch (e: any) {
      if (e.toString().includes("CooldownNotElapsed")) {
        console.log("  Очікуємо 60 секунд...");
        await new Promise(r => setTimeout(r, 62000));
        continue;
      }
      console.warn(`  Помилка пошуку: ${e.message}`);
    }

    if (searchCount < 4) {
      console.log("  Очікуємо 62 секунди...");
      await new Promise(r => setTimeout(r, 62000));
    }
  }

  // --- STEP 3: Крафт Шаблі козака ---
  console.log("\n[3] Крафт предмету 'Шабля козака'...");
  const nftMintKeypair = Keypair.generate();
  const [itemMetadataPda] = findPda(ITEM_NFT_ID, [
    Buffer.from("item_metadata"),
    nftMintKeypair.publicKey.toBuffer(),
  ]);
  const playerNftAccount = getAssociatedTokenAddressSync(
    nftMintKeypair.publicKey, payer.publicKey, false, TOKEN_2022_PROGRAM_ID
  );

  const resourceMint0 = gameConfig.resourceMints[SABER_RECIPE[0].resourceId];
  const resourceMint1 = gameConfig.resourceMints[SABER_RECIPE[1].resourceId];
  const resourceMint2 = gameConfig.resourceMints[SABER_RECIPE[2].resourceId];

  const playerResourceAccount0 = getAssociatedTokenAddressSync(resourceMint0, payer.publicKey, false, TOKEN_2022_PROGRAM_ID);
  const playerResourceAccount1 = getAssociatedTokenAddressSync(resourceMint1, payer.publicKey, false, TOKEN_2022_PROGRAM_ID);
  const playerResourceAccount2 = getAssociatedTokenAddressSync(resourceMint2, payer.publicKey, false, TOKEN_2022_PROGRAM_ID);

  await crProgram.methods
    .craftItem(0) // item_type=0: Шабля козака
    .accountsPartial({
      gameConfig: gameConfigPda,
      rmMintAuthority: mintAuthorityPda,
      craftingAuthority: craftingAuthPda,
      itemNftConfig: nftConfigPda,
      nftMintAuthority: nftMintAuthPda,
      player: payer.publicKey,
      resourceMint0,
      resourceMint1,
      resourceMint2,
      playerResourceAccount0,
      playerResourceAccount1,
      playerResourceAccount2,
      nftMint: nftMintKeypair.publicKey,
      playerNftTokenAccount: playerNftAccount,
      itemMetadata: itemMetadataPda,
      resourceManagerProgram: RESOURCE_MANAGER_ID,
      itemNftProgram: ITEM_NFT_ID,
      token2022Program: TOKEN_2022_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .signers([nftMintKeypair])
    .rpc();

  console.log(`  NFT Шабля: ${nftMintKeypair.publicKey.toBase58()}`);
  console.log(`  ItemMetadata: ${itemMetadataPda.toBase58()}`);

  // --- STEP 4: Продаж предмету ---
  console.log("\n[4] Продаж 'Шабля козака'...");
  const magicTokenMint = mtConfig.mint;
  const playerMagicTokenAccount = getAssociatedTokenAddressSync(
    magicTokenMint, payer.publicKey, false, TOKEN_2022_PROGRAM_ID
  );

  await mpProgram.methods
    .sellItem(0) // item_type=0
    .accountsPartial({
      gameConfig: gameConfigPda,
      marketplaceAuthority: marketplaceAuthPda,
      itemNftConfig: nftConfigPda,
      magicTokenConfig: mtConfigPda,
      mtMintAuthority: mtMintAuthPda,
      player: payer.publicKey,
      nftMint: nftMintKeypair.publicKey,
      itemMetadata: itemMetadataPda,
      playerNftTokenAccount: playerNftAccount,
      magicTokenMint,
      playerMagicTokenAccount,
      resourceManagerProgram: RESOURCE_MANAGER_ID,
      itemNftProgram: ITEM_NFT_ID,
      magicTokenProgram: MAGIC_TOKEN_ID,
      token2022Program: TOKEN_2022_PROGRAM_ID,
    })
    .rpc();

  // Перевіряємо баланс MagicToken
  const mtBalance = await provider.connection.getTokenAccountBalance(playerMagicTokenAccount);
  console.log(`  Баланс MagicToken: ${mtBalance.value.amount}`);
  console.log("\n✓ Тест завершено успішно! Сценарій search → craft → sell пройдено.");
}

main().catch(console.error);
