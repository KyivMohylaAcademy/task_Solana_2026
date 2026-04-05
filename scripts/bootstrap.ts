/** CLI entry point that bootstraps all shared on-chain state and writes `accounts.json`. */
import * as anchor from "@coral-xyz/anchor";
import { createWorkspace, ensureBootstrap, RESOURCE_DEFINITIONS } from "./game";

/** Reads a public-key CLI option passed as `--flag value`. */
const readPublicKeyOption = (
  flag: string,
): anchor.web3.PublicKey | undefined => {
  const index = process.argv.indexOf(flag);
  if (index === -1) {
    return undefined;
  }

  const rawValue = process.argv[index + 1];
  if (!rawValue) {
    throw new Error(`Missing value for ${flag}`);
  }

  return new anchor.web3.PublicKey(rawValue);
};

/** Runs the local bootstrap flow and prints the resulting canonical addresses. */
const main = async () => {
  const skipPlayer = process.argv.includes("--skip-player");
  const rewardTokenMint = readPublicKeyOption("--reward-mint");
  const workspace = createWorkspace();
  const result = await ensureBootstrap(workspace, {
    initializePlayer: !skipPlayer,
    writeAccountsSnapshot: true,
    rewardTokenMint,
  });

  console.log(`RPC endpoint: ${workspace.provider.connection.rpcEndpoint}`);
  console.log(`Wallet: ${workspace.walletPublicKey.toBase58()}`);
  console.log(`GameConfig: ${workspace.gameConfigPda.toBase58()}`);
  console.log(
    `Resource mints: ${RESOURCE_DEFINITIONS.map(
      (resource) =>
        `${resource.symbol}=${workspace.resourceMints[resource.resourceType].toBase58()}`,
    ).join(", ")}`,
  );
  console.log(`Default reward mint: ${workspace.magicTokenMintPda.toBase58()}`);
  console.log(
    `Configured reward mint: ${workspace.rewardTokenMint.toBase58()}`,
  );
  console.log(
    `Reward token program: ${workspace.rewardTokenProgramId.toBase58()}`,
  );
  console.log(`Player PDA: ${result.playerPda?.toBase58() ?? "skipped"}`);
  console.log(
    `Created this run: gameConfig=${result.initializedGameConfig}, resourceMints=${result.initializedResourceMints.length}, defaultRewardMint=${result.initializedDefaultRewardMint}`,
  );
  console.log("Account snapshot updated: utils/accounts.json");
};

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
