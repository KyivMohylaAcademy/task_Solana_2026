const { spawn } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const workspaceRoot = path.resolve(__dirname, "..");
const validatorLogPath = path.join(workspaceRoot, ".anchor", "validator.log");
const validatorArgs = [
  "--reset",
  "--ledger",
  ".anchor/test-ledger-8897",
  "--rpc-port",
  "8897",
  "--gossip-port",
  "8896",
  "--dynamic-port-range",
  "8890-8910",
  "--compute-unit-limit",
  "400000",
  "--faucet-port",
  "9901",
  "--bind-address",
  "0.0.0.0",
  "--url",
  "https://api.mainnet-beta.solana.com",
  "--clone-upgradeable-program",
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
  "--bpf-program",
  "5BLa56P3DuD4GjLboQGKFAVv8gsYhP5A4eHLQHxS6QLR",
  "target/deploy/crafting.so",
  "--bpf-program",
  "9uYdFs7H7iZjRYMB2r3kvGATe5ZUaKvbZrkTijR5sGEw",
  "target/deploy/item_nft.so",
  "--bpf-program",
  "CLennJDsGwnsVFAGfju1yH8frdb3QgC8i7mdXFRxhPKx",
  "target/deploy/magic_token.so",
  "--bpf-program",
  "FAuToaomcNLiYTJFVoqTiGHqakDBC3T78MNfvCiHjfNh",
  "target/deploy/marketplace.so",
  "--bpf-program",
  "4DPGoF2kGBmnt4ZJTKzB7vdzwMGvDVEBP2pbMDjtbVib",
  "target/deploy/resource_manager.so",
  "--bpf-program",
  "JCKmHHAQEmsKEi9DfS7VpiSmmGsMDszfLzGc3baFiSAW",
  "target/deploy/search.so",
];

function resolveWslGateway() {
  const routePath = "/proc/net/route";
  if (!fs.existsSync(routePath)) {
    return null;
  }

  const lines = fs.readFileSync(routePath, "utf8").trim().split("\n").slice(1);
  for (const line of lines) {
    const parts = line.trim().split(/\s+/);
    if (parts[1] !== "00000000" || !parts[2]) {
      continue;
    }

    const octets = parts[2]
      .match(/../g)
      ?.reverse()
      .map((octet) => parseInt(octet, 16));
    if (!octets || octets.length !== 4 || octets.some(Number.isNaN)) {
      continue;
    }

    return octets.join(".");
  }

  return null;
}

function resolveRpcUrl() {
  const gateway = resolveWslGateway();
  if (gateway) {
    return `http://${gateway}:8897`;
  }
  return "http://127.0.0.1:8897";
}

const rpcUrl = resolveRpcUrl();

function resolvePowerShellBinary() {
  const candidates = [
    "/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe",
    "powershell.exe",
  ];

  return candidates.find((candidate) => {
    if (candidate === "powershell.exe") {
      return true;
    }
    return fs.existsSync(candidate);
  });
}

async function killValidatorOnPort(port) {
  const powerShell = resolvePowerShellBinary();
  if (!powerShell) {
    return;
  }

  await new Promise((resolve) => {
    const command = [
      "$conn = Get-NetTCPConnection -LocalPort",
      String(port),
      "-State Listen -ErrorAction SilentlyContinue | Select-Object -First 1;",
      "if ($conn) { Stop-Process -Id $conn.OwningProcess -Force -ErrorAction SilentlyContinue }",
    ].join(" ");

    const proc = spawn(
      powerShell,
      ["-NoProfile", "-Command", command],
      { stdio: "ignore", shell: false }
    );

    proc.on("exit", () => resolve());
    proc.on("error", () => resolve());
  });
}

function resolveValidatorBinary() {
  const windowsHome =
    process.env.USERPROFILE ||
    (process.platform === "win32" ? os.homedir() : "C:\\Users\\artem");
  const wslWindowsHome = windowsHome
    .replace(/\\/g, "/")
    .replace(/^([A-Za-z]):/, (_, drive) => `/mnt/${drive.toLowerCase()}`);
  const candidates = [
    process.env.SOLANA_TEST_VALIDATOR,
    path.join(os.homedir(), ".cargo", "bin", "solana-test-validator.exe"),
    path.join(os.homedir(), ".cargo", "bin", "solana-test-validator"),
    path.posix.join(wslWindowsHome, ".cargo", "bin", "solana-test-validator.exe"),
    path.posix.join(
      wslWindowsHome,
      "agave",
      "v2.3.13",
      "solana-release",
      "bin",
      "solana-test-validator.exe"
    ),
    path.join(
      windowsHome,
      ".cargo",
      "bin",
      "solana-test-validator.exe"
    ),
    path.join(
      windowsHome,
      "agave",
      "v2.3.13",
      "solana-release",
      "bin",
      "solana-test-validator.exe"
    ),
    process.platform === "win32"
      ? "solana-test-validator"
      : "/mnt/c/Users/artem/.cargo/bin/solana-test-validator.exe",
    "solana-test-validator",
  ].filter(Boolean);

  const existing = candidates.find((candidate) => {
    if (candidate === "solana-test-validator") {
      return false;
    }
    return fs.existsSync(candidate);
  });

  return existing ?? "solana-test-validator";
}

async function isRpcReady() {
  try {
    const response = await fetch(rpcUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "getLatestBlockhash",
        params: [{ commitment: "processed" }],
      }),
    });

    if (!response.ok) {
      return false;
    }

    const body = await response.json();
    return Boolean(body.result?.value?.blockhash);
  } catch {
    return false;
  }
}

async function waitForRpc(timeoutMs) {
  const startedAt = Date.now();
  while (Date.now() - startedAt < timeoutMs) {
    if (await isRpcReady()) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }
  throw new Error(`Local validator did not become ready at ${rpcUrl}`);
}

async function main() {
  let validatorProcess = null;
  await killValidatorOnPort(8897);

  const validatorBinary = resolveValidatorBinary();
  fs.mkdirSync(path.dirname(validatorLogPath), { recursive: true });
  const validatorLogStream = fs.createWriteStream(validatorLogPath, {
    flags: "w",
  });
  validatorProcess = spawn(validatorBinary, validatorArgs, {
    cwd: workspaceRoot,
    stdio: ["ignore", "pipe", "pipe"],
    shell: false,
  });

  validatorProcess.stdout.pipe(validatorLogStream);
  validatorProcess.stderr.pipe(validatorLogStream);

  validatorProcess.on("error", (error) => {
    console.error(error);
    process.exit(1);
  });

  await waitForRpc(90000);

  const mochaProcess = spawn(
    "npx",
    ["ts-mocha", "-p", "./tsconfig.json", "-t", "1000000", "tests/**/*.ts"],
    {
      cwd: workspaceRoot,
      stdio: "inherit",
      shell: true,
      env: {
        ...process.env,
        ANCHOR_PROVIDER_URL: rpcUrl,
      },
    }
  );

  const stopValidator = () => {
    if (validatorProcess && !validatorProcess.killed) {
      validatorProcess.kill("SIGTERM");
    }
    validatorLogStream.end();
  };

  process.on("SIGINT", () => {
    stopValidator();
    process.exit(130);
  });

  process.on("SIGTERM", () => {
    stopValidator();
    process.exit(143);
  });

  mochaProcess.on("exit", (code) => {
    stopValidator();
    process.exit(code ?? 1);
  });
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
