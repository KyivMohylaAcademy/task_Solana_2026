/**
 * Білд та деплой всіх 6 програм на Solana Devnet.
 * Запуск: npm run deploy
 */

import { execSync, spawnSync } from "child_process";
import * as fs from "fs";
import * as path from "path";

const ROOT = path.join(__dirname, "..");

const PROGRAMS: Array<{
  name: string;
  binaryName: string;
  label: string;
}> = [
  { name: "resource_manager", binaryName: "resource_manager", label: "resource_manager" },
  { name: "magic_token",      binaryName: "magic_token",      label: "magic_token" },
  { name: "item_nft",         binaryName: "item_nft",         label: "item_nft" },
  { name: "search",           binaryName: "search",           label: "search" },
  { name: "crafting",         binaryName: "crafting",         label: "crafting" },
  { name: "marketplace",      binaryName: "marketplace",      label: "marketplace" },
];

function run(cmd: string, opts: { cwd?: string; silent?: boolean } = {}) {
  const cwd = opts.cwd ?? ROOT;
  if (!opts.silent) console.log(`  $ ${cmd}`);
  try {
    const out = execSync(cmd, { cwd, stdio: opts.silent ? "pipe" : "inherit", encoding: "utf-8" });
    return out?.trim() ?? "";
  } catch (err: any) {
    throw new Error(err.stderr?.toString() ?? err.message);
  }
}

function getProgramId(binaryName: string): string | null {
  const keypairPath = path.join(ROOT, "target", "deploy", `${binaryName}-keypair.json`);
  // Перевіряємо keypair у programs/<name>/target/deploy/ (стара структура)
  const altKeypairPath = path.join(ROOT, "programs", binaryName, "target", "deploy", `${binaryName}-keypair.json`);

  const kpPath = fs.existsSync(keypairPath) ? keypairPath
               : fs.existsSync(altKeypairPath) ? altKeypairPath
               : null;

  if (!kpPath) return null;

  try {
    const raw = JSON.parse(fs.readFileSync(kpPath, "utf-8")) as number[];
    const { PublicKey } = require("@solana/web3.js");
    const kp = require("@solana/web3.js").Keypair.fromSecretKey(Uint8Array.from(raw));
    return kp.publicKey.toBase58();
  } catch {
    return null;
  }
}

function log(msg: string) {
  console.log(`[deploy] ${msg}`);
}

async function main() {
  console.log("═══════════════════════════════════════════════");
  console.log("  Козацький бізнес — Деплой на Devnet          ");
  console.log("═══════════════════════════════════════════════\n");

  log("Перевіряємо конфігурацію Solana CLI...");
  const solanaConfig = run("solana config get", { silent: true });
  const clusterMatch = solanaConfig.match(/RPC URL:\s+(.+)/);
  const cluster = clusterMatch?.[1]?.trim() ?? "unknown";
  log(`Кластер: ${cluster}`);

  if (!cluster.includes("devnet")) {
    console.warn("\n⚠ Кластер не devnet. Встановлюємо devnet...");
    run("solana config set --url devnet");
  }

  const walletOut = run("solana address", { silent: true });
  const adminPubkey = walletOut.trim();
  log(`Гаманець: ${adminPubkey}`);

  const balanceOut = run("solana balance --url devnet", { silent: true });
  log(`Баланс:   ${balanceOut}`);

  console.log("\n─── Збірка програм ───────────────────────────");
  run("anchor build --skip-lint");
  log("✔ Збірка завершена.");

  console.log("\n─── Деплой програм ───────────────────────────");

  const deployedIds: Record<string, string> = {};

  for (const prog of PROGRAMS) {
    const soPath = path.join(
      ROOT, "target", "sbpf-solana-solana", "release", `${prog.binaryName}.so`
    );

    if (!fs.existsSync(soPath)) {
      console.error(`✗ Не знайдено .so: ${soPath}`);
      continue;
    }

    const keypairPath = path.join(ROOT, "target", "deploy", `${prog.binaryName}-keypair.json`);
    const altKeypairPath = path.join(
      ROOT, "programs", prog.name, "target", "deploy", `${prog.binaryName}-keypair.json`
    );
    const kpPath = fs.existsSync(keypairPath) ? keypairPath
                 : fs.existsSync(altKeypairPath) ? altKeypairPath
                 : null;

    let deployCmd: string;
    if (kpPath) {
      deployCmd = `solana program deploy --program-id "${kpPath}" "${soPath}" --url devnet`;
    } else {
      deployCmd = `solana program deploy "${soPath}" --url devnet`;
    }

    process.stdout.write(`  Деплой ${prog.label}... `);
    try {
      const out = run(deployCmd, { silent: true });
      const idMatch = out.match(/Program Id:\s+(\S+)/);
      const programId = idMatch?.[1] ?? getProgramId(prog.binaryName) ?? "невідомо";
      deployedIds[prog.label] = programId;
      console.log(`✔ ${programId}`);
    } catch (err: any) {
      console.log(`✗`);
      console.error(`    Помилка: ${err.message?.split("\n")[0]}`);
    }
  }

  console.log("\n─── Результат деплою ─────────────────────────");
  console.log("Program IDs (Devnet):\n");
  for (const [name, id] of Object.entries(deployedIds)) {
    console.log(`  ${name.padEnd(20)} ${id}`);
  }

  // Зберігаємо у файл
  const result = {
    network: "devnet",
    rpcUrl: "https://api.devnet.solana.com",
    deployedAt: new Date().toISOString(),
    programs: deployedIds,
  };
  const outPath = path.join(ROOT, "deploy-result.json");
  fs.writeFileSync(outPath, JSON.stringify(result, null, 2));
  log(`\nРезультати збережено: deploy-result.json`);

  console.log("\n✔ Деплой завершено!");
  console.log("  Наступний крок: npm run setup");
}

main().catch((err) => {
  console.error("\nПомилка деплою:", err.message ?? err);
  process.exit(1);
});
