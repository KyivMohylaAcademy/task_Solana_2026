/**
 * Демо взаємодії з грою: search → craft → list → buy.
 * Запуск: npm run interact
 */

import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  getAccount,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
  Connection,
  Transaction,
  SystemProgram,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

import { ResourceManager } from "../target/types/resource_manager";
import { MagicToken } from "../target/types/magic_token";
import { ItemNft } from "../target/types/item_nft";
import { Search } from "../target/types/search";
import { Crafting } from "../target/types/crafting";
import { Marketplace } from "../target/types/marketplace";

const DEVNET_URL = "https://api.devnet.solana.com";
const WALLET_PATH = path.join(os.homedir(), ".config", "solana", "id.json");

const MPL_TOKEN_METADATA_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);
const SYSVAR_INSTRUCTIONS_PUBKEY = new PublicKey(
  "Sysvar1nstructions1111111111111111111111111"
);

function loadWallet(walletPath: string): Keypair {
  const raw = fs.readFileSync(walletPath, "utf-8");
  return Keypair.fromSecretKey(Uint8Array.from(JSON.parse(raw)));
}

function mplMetaPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), MPL_TOKEN_METADATA_ID.toBuffer(), mint.toBuffer()],
    MPL_TOKEN_METADATA_ID
  )[0];
}

function mplMasterEditionPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from("edition"),
    ],
    MPL_TOKEN_METADATA_ID
  )[0];
}

function ata22(mint: PublicKey, owner: PublicKey): PublicKey {
  const [addr] = PublicKey.findProgramAddressSync(
    [
      owner.toBuffer(),
      TOKEN_2022_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  return addr;
}

function itemMetaPda(nftMint: PublicKey, itemNftProgramId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("item_metadata"), nftMint.toBuffer()],
    itemNftProgramId
  )[0];
}

function step(n: number, msg: string) {
  console.log(`\n[${n}] ${msg}`);
  console.log("─".repeat(50));
}

async function main() {
  console.log("═══════════════════════════════════════════════════");
  console.log("  Козацький бізнес — Демо взаємодії (interact.ts)  ");
  console.log("═══════════════════════════════════════════════════");

  // Завантажуємо результати setup
  const setupPath = path.join(__dirname, "..", "setup-result.json");
  if (!fs.existsSync(setupPath)) {
    console.error("Спочатку запустіть: npm run setup");
    process.exit(1);
  }
  const setupResult = JSON.parse(fs.readFileSync(setupPath, "utf-8"));
  const resourceMints = Object.values(setupResult.resourceMints).map(
    (addr) => new PublicKey(addr as string)
  );

  // Провайдер
  const connection = new Connection(DEVNET_URL, "confirmed");
  const admin = loadWallet(WALLET_PATH);
  const wallet = new anchor.Wallet(admin);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "processed",
  });
  anchor.setProvider(provider);

  // Програми
  const rmProg  = anchor.workspace.ResourceManager as anchor.Program<ResourceManager>;
  const mtProg  = anchor.workspace.MagicToken      as anchor.Program<MagicToken>;
  const nftProg = anchor.workspace.ItemNft         as anchor.Program<ItemNft>;
  const srProg  = anchor.workspace.Search          as anchor.Program<Search>;
  const crProg  = anchor.workspace.Crafting        as anchor.Program<Crafting>;
  const mpProg  = anchor.workspace.Marketplace     as anchor.Program<Marketplace>;

  // Гравець (новий keypair для демо)
  const player = Keypair.generate();
  console.log(`\nГравець: ${player.publicKey.toBase58()}`);
  console.log(`Admin:   ${admin.publicKey.toBase58()}`);

  // Фінансуємо гравця з admin
  const fundTx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: admin.publicKey,
      toPubkey: player.publicKey,
      lamports: 0.3 * LAMPORTS_PER_SOL,
    })
  );
  await provider.sendAndConfirm(fundTx, [], { commitment: "confirmed" });
  console.log("✔ Гравцю відправлено 0.3 SOL");

  // PDAs
  const [gameConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("game_config")],
    rmProg.programId
  );
  const [magicMintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("magic_mint")],
    mtProg.programId
  );
  const [playerPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("player"), player.publicKey.toBuffer()],
    srProg.programId
  );

  // Створюємо ATA гравця для кожного ресурсу
  const playerTas: PublicKey[] = [];
  for (let i = 0; i < 6; i++) {
    const ta = await getOrCreateAssociatedTokenAccount(
      connection, player,
      resourceMints[i], player.publicKey,
      false, "confirmed", { commitment: "confirmed" },
      TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID
    );
    playerTas.push(ta.address);
  }

  // ── Крок 1: Ініціалізація гравця ──────────────────────────────────────────
  step(1, "Ініціалізація Player PDA (search program)");

  await srProg.methods
    .initializePlayer()
    .accounts({ owner: player.publicKey })
    .signers([player])
    .rpc({ commitment: "confirmed" });

  const playerState = await srProg.account.player.fetch(playerPda);
  console.log(`✔ Player PDA: ${playerPda.toBase58()}`);
  console.log(`  owner:               ${playerState.owner.toBase58()}`);
  console.log(`  lastSearchTimestamp: ${playerState.lastSearchTimestamp.toNumber()}`);

  // ── Крок 2: Пошук ресурсів ────────────────────────────────────────────────
  step(2, "Пошук ресурсів (search_resources)");

  const remaining = [];
  for (let i = 0; i < 6; i++) {
    remaining.push({ pubkey: resourceMints[i], isWritable: true, isSigner: false });
    remaining.push({ pubkey: playerTas[i],     isWritable: true, isSigner: false });
  }

  let totalBefore = BigInt(0);
  for (const ta of playerTas) {
    const acc = await getAccount(connection, ta, "confirmed", TOKEN_2022_PROGRAM_ID);
    totalBefore += acc.amount;
  }

  await srProg.methods
    .searchResources()
    .accounts({ owner: player.publicKey })
    .remainingAccounts(remaining)
    .signers([player])
    .rpc({ commitment: "confirmed" });

  let totalAfter = BigInt(0);
  const resourceNames = ["WOOD", "IRON", "GOLD", "LEATHER", "STONE", "DIAMOND"];
  for (let i = 0; i < 6; i++) {
    const acc = await getAccount(connection, playerTas[i], "confirmed", TOKEN_2022_PROGRAM_ID);
    totalAfter += acc.amount;
    if (acc.amount > BigInt(0)) {
      console.log(`  ${resourceNames[i]}: +${acc.amount}`);
    }
  }
  console.log(`✔ Знайдено ресурсів: ${totalAfter - totalBefore} (завжди 3)`);

  // ── Крок 3: Мінт ресурсів для рецепту ────────────────────────────────────
  step(3, "Мінт ресурсів для рецепту Шаблі козака (3×IRON + 1×WOOD + 1×LEATHER)");
  // Рецепт item_0: [(IRON,3),(WOOD,1),(LEATHER,1)]
  // Мінтимо додатково щоб гарантувати достатній баланс

  async function mintRes(id: number, amount: number) {
    await rmProg.methods
      .mintResource(id, new BN(amount))
      .accounts({
        resourceMint:       resourceMints[id],
        playerTokenAccount: playerTas[id],
        player:             player.publicKey,
      })
      .rpc({ commitment: "confirmed" });
  }

  await mintRes(1, 3); // IRON
  await mintRes(0, 1); // WOOD
  await mintRes(3, 1); // LEATHER

  console.log("✔ IRON:    +3");
  console.log("✔ WOOD:    +1");
  console.log("✔ LEATHER: +1");

  // ── Крок 4: Крафт Шаблі козака ────────────────────────────────────────────
  step(4, "Крафт Шаблі козака (crafting → item_nft CPI)");

  const nftKp  = Keypair.generate();
  const nftAta = ata22(nftKp.publicKey, player.publicKey);

  const craftRemaining = [
    { pubkey: resourceMints[1], isWritable: true, isSigner: false }, // IRON mint
    { pubkey: playerTas[1],     isWritable: true, isSigner: false }, // IRON ta
    { pubkey: resourceMints[0], isWritable: true, isSigner: false }, // WOOD mint
    { pubkey: playerTas[0],     isWritable: true, isSigner: false }, // WOOD ta
    { pubkey: resourceMints[3], isWritable: true, isSigner: false }, // LEATHER mint
    { pubkey: playerTas[3],     isWritable: true, isSigner: false }, // LEATHER ta
  ];

  await crProg.methods
    .craftItem(0, "https://arweave.net/saber-demo")
    .accounts({
      player:             player.publicKey,
      itemMetadata:       itemMetaPda(nftKp.publicKey, nftProg.programId),
      nftMint:            nftKp.publicKey,
      playerNftAccount:   nftAta,
      metadataAccount:    mplMetaPda(nftKp.publicKey),
      masterEdition:      mplMasterEditionPda(nftKp.publicKey),
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
    })
    .preInstructions([
      ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
    ])
    .remainingAccounts(craftRemaining)
    .signers([player, nftKp])
    .rpc({ commitment: "confirmed" });

  const nftAcct = await getAccount(connection, nftAta, "confirmed", TOKEN_2022_PROGRAM_ID);
  console.log(`✔ NFT mint:   ${nftKp.publicKey.toBase58()}`);
  console.log(`✔ NFT ATA:    ${nftAta.toBase58()}`);
  console.log(`✔ Баланс ATA: ${nftAcct.amount} (має бути 1)`);

  // ── Крок 5: Виставлення на маркетплейс ────────────────────────────────────
  step(5, "Виставлення Шаблі на маркетплейс за 5 MagicToken");

  const [listingPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("listing"), nftKp.publicKey.toBuffer()],
    mpProg.programId
  );
  const [escrowPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), nftKp.publicKey.toBuffer()],
    mpProg.programId
  );

  await mpProg.methods
    .listItem(new BN(5))
    .accounts({
      seller:           player.publicKey,
      nftMint:          nftKp.publicKey,
      sellerNftAccount: nftAta,
      tokenProgram:     TOKEN_2022_PROGRAM_ID,
    })
    .signers([player])
    .rpc({ commitment: "confirmed" });

  const listing = await mpProg.account.listing.fetch(listingPda);
  const escrow  = await getAccount(connection, escrowPda, "confirmed", TOKEN_2022_PROGRAM_ID);
  console.log(`✔ Listing PDA:  ${listingPda.toBase58()}`);
  console.log(`✔ Ціна:         ${listing.price.toNumber()} MagicToken`);
  console.log(`✔ NFT в ескроу: ${escrow.amount}`);

  // ── Крок 6: Купівля покупцем ──────────────────────────────────────────────
  step(6, "Купівля NFT покупцем → гравець отримує MagicToken");

  // Створюємо ATA продавця для MagicToken
  const sellerMagicAtaObj = await getOrCreateAssociatedTokenAccount(
    connection, player,
    magicMintPda, player.publicKey,
    false, "confirmed", { commitment: "confirmed" },
    TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID
  );
  const sellerMagicAta = sellerMagicAtaObj.address;

  const magicBefore = (
    await getAccount(connection, sellerMagicAta, "confirmed", TOKEN_2022_PROGRAM_ID)
  ).amount;

  // Покупець — admin (для простоти, бо вже має SOL)
  // У реальній грі це був би інший гравець
  const buyer = admin;

  await mpProg.methods
    .buyItem()
    .accounts({
      buyer:              buyer.publicKey,
      seller:             player.publicKey,
      nftMint:            nftKp.publicKey,
      itemMetadata:       itemMetaPda(nftKp.publicKey, nftProg.programId),
      metadataAccount:    mplMetaPda(nftKp.publicKey),
      masterEdition:      mplMasterEditionPda(nftKp.publicKey),
      magicMint:          magicMintPda,
      sellerMagicAccount: sellerMagicAta,
      sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
    })
    .signers([buyer])
    .rpc({ commitment: "confirmed" });

  const magicAfter = (
    await getAccount(connection, sellerMagicAta, "confirmed", TOKEN_2022_PROGRAM_ID)
  ).amount;

  console.log(`✔ Продавець отримав: ${magicAfter - magicBefore} MagicToken`);
  console.log(`✔ Загальний баланс MagicToken: ${magicAfter}`);

  // NFT спалено — ItemMetadata закрита
  const metaAcct = await connection.getAccountInfo(
    itemMetaPda(nftKp.publicKey, nftProg.programId)
  );
  console.log(`✔ ItemMetadata закрита: ${metaAcct === null ? "так" : "ні"}`);

  // ── Підсумок ──────────────────────────────────────────────────────────────
  console.log("\n" + "═".repeat(51));
  console.log("  Демо завершено успішно!");
  console.log("  Флоу: search → craft → list → buy — OK");
  console.log("═".repeat(51));
}

main().catch((err) => {
  console.error("\nПомилка:", err);
  process.exit(1);
});
