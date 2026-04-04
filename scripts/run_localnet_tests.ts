import { spawn, type ChildProcess } from "node:child_process";
import { mkdirSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";

const METADATA_PROGRAM_ID = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
const RPC_PORT = 18999;
const FAUCET_PORT = 19999;
const GOSSIP_PORT = 18000;
const METAPLEX_SOURCE_URL = process.env.METAPLEX_SOURCE_URL ?? "mainnet-beta";

const projectRoot = process.cwd();
const ledgerPath = path.join(projectRoot, ".anchor", "test-ledger");
const rpcUrl = `http://127.0.0.1:${RPC_PORT}`;
const LOCALNET_DEPLOYER_AIRDROP_SOL = 100;

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

async function waitForValidator(url: string, attempts = 90, delayMs = 1000) {
  const connection = new Connection(url, "confirmed");
  let lastError: unknown;
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      await connection.getVersion();
      return;
    } catch (error) {
      lastError = error;
      await new Promise((resolve) => setTimeout(resolve, delayMs));
    }
  }
  throw lastError ?? new Error("Validator did not become ready");
}

async function fundWallet(url: string, walletPath: string) {
  const connection = new Connection(url, "confirmed");
  const secretKey = Uint8Array.from(JSON.parse(readFileSync(walletPath, "utf8")) as number[]);
  const wallet = Keypair.fromSecretKey(secretKey);
  const signature = await connection.requestAirdrop(
    wallet.publicKey,
    LOCALNET_DEPLOYER_AIRDROP_SOL * LAMPORTS_PER_SOL,
  );
  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction(
    {
      signature,
      ...latestBlockhash,
    },
    "confirmed",
  );
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

async function run() {
  mkdirSync(path.join(projectRoot, ".anchor"), { recursive: true });
  rmSync(ledgerPath, { recursive: true, force: true });
  const walletPath = resolveWalletPath(projectRoot);

  const build = spawnOrThrow("anchor", ["build"], projectRoot);
  await waitForExit(build, "anchor build");

  const validatorArgs = [
    "--ledger",
    ledgerPath,
    "--reset",
    "--bind-address",
    "127.0.0.1",
    "--rpc-port",
    String(RPC_PORT),
    "--faucet-port",
    String(FAUCET_PORT),
    "--gossip-port",
    String(GOSSIP_PORT),
    "--url",
    METAPLEX_SOURCE_URL,
    "--clone-upgradeable-program",
    METADATA_PROGRAM_ID,
  ];
  const validator = spawnOrThrow("solana-test-validator", validatorArgs, projectRoot);

  const stopValidator = () => {
    if (!validator.killed) {
      validator.kill("SIGINT");
    }
  };

  process.on("SIGINT", () => {
    stopValidator();
    process.exit(130);
  });
  process.on("SIGTERM", () => {
    stopValidator();
    process.exit(143);
  });

  try {
    await waitForValidator(rpcUrl);
    await fundWallet(rpcUrl, walletPath);
    const test = spawnOrThrow(
      "anchor",
      [
        "test",
        "--skip-build",
        "--skip-local-validator",
        "--provider.cluster",
        rpcUrl,
      ],
      projectRoot,
    );
    await waitForExit(test, "anchor test");
  } finally {
    stopValidator();
  }
}

run().catch((error) => {
  console.error(error);
  process.exit(1);
});
