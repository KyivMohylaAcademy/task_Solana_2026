import { spawn, type ChildProcess } from "node:child_process";
import { readFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { Connection, Keypair, LAMPORTS_PER_SOL, clusterApiUrl } from "@solana/web3.js";

const projectRoot = process.cwd();
const DEVNET_URL = "devnet";
const MIN_DEPLOY_BALANCE_SOL = 15;
const DEVNET_AIRDROP_CHUNK_SOL = 5;

function resolveWalletPath(projectRootDir: string) {
  const anchorToml = readFileSync(path.join(projectRootDir, "Anchor.toml"), "utf8");
  const match = anchorToml.match(/^\s*wallet\s*=\s*"([^"]+)"\s*$/m);
  const configuredPath = process.env.ANCHOR_WALLET ?? match?.[1];
  if (!configuredPath) {
    throw new Error("Unable to resolve wallet path from Anchor.toml");
  }
  if (configuredPath.startsWith("~/")) {
    return path.join(os.homedir(), configuredPath.slice(2));
  }
  if (path.isAbsolute(configuredPath)) {
    return configuredPath;
  }
  return path.resolve(projectRootDir, configuredPath);
}

function loadWallet(walletPath: string) {
  const secretKey = Uint8Array.from(JSON.parse(readFileSync(walletPath, "utf8")) as number[]);
  return Keypair.fromSecretKey(secretKey);
}

function spawnOrThrow(command: string, args: string[], cwd: string) {
  const child = spawn(command, args, {
    cwd,
    stdio: "inherit",
    env: process.env,
  });
  child.on("error", (error) => {
    console.error(`Failed to start ${command}:`, error);
  });
  return child;
}

function waitForExit(child: ChildProcess, label: string) {
  return new Promise<void>((resolve, reject) => {
    child.once("exit", (code, signal) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(
        new Error(
          `${label} exited with code ${code ?? "null"} and signal ${signal ?? "null"}`,
        ),
      );
    });
    child.once("error", reject);
  });
}

async function ensureDevnetBalance(walletPath: string) {
  const connection = new Connection(clusterApiUrl(DEVNET_URL), "confirmed");
  const wallet = loadWallet(walletPath);
  const targetLamports = MIN_DEPLOY_BALANCE_SOL * LAMPORTS_PER_SOL;
  let balanceLamports = await connection.getBalance(wallet.publicKey);

  console.log(`Deploy wallet: ${wallet.publicKey.toBase58()}`);
  console.log(`Devnet balance: ${(balanceLamports / LAMPORTS_PER_SOL).toFixed(4)} SOL`);

  while (balanceLamports < targetLamports) {
    const missingLamports = targetLamports - balanceLamports;
    const requestLamports = Math.min(
      missingLamports,
      DEVNET_AIRDROP_CHUNK_SOL * LAMPORTS_PER_SOL,
    );

    console.log(
      `Requesting airdrop: ${(requestLamports / LAMPORTS_PER_SOL).toFixed(4)} SOL`,
    );

    try {
      const signature = await connection.requestAirdrop(wallet.publicKey, requestLamports);
      const latestBlockhash = await connection.getLatestBlockhash();
      await connection.confirmTransaction(
        {
          signature,
          ...latestBlockhash,
        },
        "confirmed",
      );
      balanceLamports = await connection.getBalance(wallet.publicKey);
      console.log(`Updated balance: ${(balanceLamports / LAMPORTS_PER_SOL).toFixed(4)} SOL`);
    } catch (error) {
      throw new Error(
        `Devnet airdrop failed for ${wallet.publicKey.toBase58()}. Fund it manually and rerun deploy. ${String(error)}`,
      );
    }
  }
}

async function run() {
  const walletPath = resolveWalletPath(projectRoot);
  const setConfig = spawnOrThrow("solana", ["config", "set", "--url", DEVNET_URL], projectRoot);
  await waitForExit(setConfig, "solana config set");
  await ensureDevnetBalance(walletPath);

  const build = spawnOrThrow("anchor", ["build"], projectRoot);
  await waitForExit(build, "anchor build");

  const deploy = spawnOrThrow(
    "anchor",
    ["deploy", "--provider.cluster", "devnet"],
    projectRoot,
  );
  await waitForExit(deploy, "anchor deploy");
}

run().catch((error) => {
  console.error(error);
  process.exit(1);
});
