/** CLI helper that ensures only mint-related bootstrap state exists and prints addresses. */
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

/** Creates missing resource and reward mints and prints their addresses. */
const main = async () => {
  const rewardTokenMint = readPublicKeyOption("--reward-mint");
  const workspace = createWorkspace();
  const result = await ensureBootstrap(workspace, {
    initializePlayer: false,
    writeAccountsSnapshot: true,
    rewardTokenMint,
  });

  console.log(`RPC endpoint: ${workspace.provider.connection.rpcEndpoint}`);
  console.log(`GameConfig: ${workspace.gameConfigPda.toBase58()}`);
  for (const resource of RESOURCE_DEFINITIONS) {
    console.log(
      `${resource.name} mint: ${workspace.resourceMints[resource.resourceType].toBase58()}`,
    );
  }
  console.log(`Default reward mint: ${workspace.magicTokenMintPda.toBase58()}`);
  console.log(
    `Configured reward mint: ${workspace.rewardTokenMint.toBase58()}`,
  );
  console.log(
    `Reward token program: ${workspace.rewardTokenProgramId.toBase58()}`,
  );
  console.log(
    `Created this run: gameConfig=${result.initializedGameConfig}, resourceMints=${result.initializedResourceMints.join(",") || "none"}, defaultRewardMint=${result.initializedDefaultRewardMint}`,
  );
};

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
