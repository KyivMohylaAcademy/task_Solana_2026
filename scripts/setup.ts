/**
 * Ініціалізація "Козацький бізнес" на Solana Devnet.
 * Запуск: npm run setup
 */

import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
  Connection,
} from "@solana/web3.js";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

import { ResourceManager } from "../target/types/resource_manager";
import { MagicToken } from "../target/types/magic_token";

const DEVNET_URL = "https://api.devnet.solana.com";
const WALLET_PATH = path.join(os.homedir(), ".config", "solana", "id.json");

const RESOURCE_NAMES = ["WOOD", "IRON", "GOLD", "LEATHER", "STONE", "DIAMOND"];

function loadWallet(walletPath: string): Keypair {
  const raw = fs.readFileSync(walletPath, "utf-8");
  const secretKey = Uint8Array.from(JSON.parse(raw));
  return Keypair.fromSecretKey(secretKey);
}

function log(msg: string) {
  console.log(`[setup] ${msg}`);
}

async function main() {
  console.log("═══════════════════════════════════════════════");
  console.log("  Козацький бізнес — Ініціалізація (setup.ts)  ");
  console.log("═══════════════════════════════════════════════\n");

  // 1. З'єднання та провайдер
  const connection = new Connection(DEVNET_URL, "confirmed");
  const admin = loadWallet(WALLET_PATH);
  const wallet = new anchor.Wallet(admin);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "processed",
  });
  anchor.setProvider(provider);

  const balance = await connection.getBalance(admin.publicKey);
  log(`Гаманець: ${admin.publicKey.toBase58()}`);
  log(`Баланс:   ${(balance / LAMPORTS_PER_SOL).toFixed(4)} SOL`);

  if (balance < 0.1 * LAMPORTS_PER_SOL) {
    console.error("\n⚠ Недостатньо SOL. Поповніть: https://faucet.solana.com/");
    process.exit(1);
  }

  // Завантажуємо програми
  const rmProg = anchor.workspace.ResourceManager as anchor.Program<ResourceManager>;
  const mtProg = anchor.workspace.MagicToken as anchor.Program<MagicToken>;

  log(`resource_manager: ${rmProg.programId.toBase58()}`);
  log(`magic_token:      ${mtProg.programId.toBase58()}\n`);

  // PDAs
  const [magicMintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("magic_mint")],
    mtProg.programId
  );
  const [gameConfigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("game_config")],
    rmProg.programId
  );
  const [marketplaceAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("marketplace_authority")],
    // marketplace program id з Anchor.toml
    new PublicKey("DuCW5GarF2Z6Jb2SXTL42hViJ6SwdPvKJFJkq6wtP5eC")
  );

  const magicConfigInfo = await connection.getAccountInfo(
    PublicKey.findProgramAddressSync(
      [Buffer.from("magic_config")],
      mtProg.programId
    )[0],
    "confirmed"
  );

  if (magicConfigInfo) {
    log("MagicToken вже ініціалізовано — пропускаємо.");
  } else {
    log("Ініціалізуємо MagicToken...");
    await mtProg.methods
      .initializeMagicToken()
      .accounts({ marketplaceAuthority })
      .signers([admin])
      .rpc({ commitment: "confirmed" });
    log(`✔ MagicMint PDA: ${magicMintPda.toBase58()}`);
  }

  const gameConfigInfo = await connection.getAccountInfo(gameConfigPda, "confirmed");

  let resourceMints: PublicKey[];

  if (gameConfigInfo) {
    log("GameConfig вже ініціалізовано — читаємо існуючі мінти.");
    const cfg = await rmProg.account.gameConfig.fetch(gameConfigPda);
    resourceMints = cfg.resourceMints as PublicKey[];
    for (let i = 0; i < 6; i++) {
      log(`  ${RESOURCE_NAMES[i]}: ${resourceMints[i].toBase58()}`);
    }
  } else {
    log("Створюємо 6 SPL Token-2022 мінтів ресурсів...");
    resourceMints = [];
    for (let i = 0; i < 6; i++) {
      const mint = await createMint(
        connection,
        admin,
        gameConfigPda,  // mint_authority = GameConfig PDA
        null,
        0,
        undefined,
        { commitment: "confirmed" },
        TOKEN_2022_PROGRAM_ID
      );
      resourceMints.push(mint);
      log(`  ✔ ${RESOURCE_NAMES[i]}: ${mint.toBase58()}`);
    }

    log("\nІніціалізуємо GameConfig...");
    await rmProg.methods
      .initializeGameConfig()
      .accounts({
        mintWood:       resourceMints[0],
        mintIron:       resourceMints[1],
        mintGold:       resourceMints[2],
        mintLeather:    resourceMints[3],
        mintStone:      resourceMints[4],
        mintDiamond:    resourceMints[5],
        magicTokenMint: magicMintPda,
      })
      .signers([admin])
      .rpc({ commitment: "confirmed" });
    log("✔ GameConfig ініціалізовано.");
  }

  const setupResult = {
    network: DEVNET_URL,
    admin: admin.publicKey.toBase58(),
    programs: {
      resource_manager: rmProg.programId.toBase58(),
      magic_token:      mtProg.programId.toBase58(),
      item_nft:         "EEvis5VXdC5NRn4BrfUYdwcpzgptrfJZ3PsZ2UVxU8WY",
      crafting:         "BqUaswbtzh21TNbcCpB2TnZEsVy6W3dAwgNZhDKEpc44",
      search:           "3BRTKJk5xnswDXDMKogS43hcJVdpr1XDJrStZyLGduvs",
      marketplace:      "DuCW5GarF2Z6Jb2SXTL42hViJ6SwdPvKJFJkq6wtP5eC",
    },
    pdas: {
      gameConfig:          gameConfigPda.toBase58(),
      magicMint:           magicMintPda.toBase58(),
      marketplaceAuthority: marketplaceAuthority.toBase58(),
    },
    resourceMints: Object.fromEntries(
      RESOURCE_NAMES.map((name, i) => [name, resourceMints[i].toBase58()])
    ),
  };

  const outPath = path.join(__dirname, "..", "setup-result.json");
  fs.writeFileSync(outPath, JSON.stringify(setupResult, null, 2));

  console.log("\n═══════════════════════════════════════════");
  console.log("  Ініціалізація завершена!");
  console.log(`  Адреси збережено: setup-result.json`);
  console.log("═══════════════════════════════════════════");
}

main().catch((err) => {
  console.error("Помилка:", err);
  process.exit(1);
});
