/** Anchor deploy hook that bootstraps shared state after program deployment. */
import * as anchor from "@coral-xyz/anchor";
import { createWorkspace, ensureBootstrap } from "../scripts/game";

/** Ensures the deployed programs have the shared config and mint state they require. */
const deploy = async (provider: anchor.AnchorProvider) => {
  const workspace = createWorkspace(provider);

  await ensureBootstrap(workspace, {
    initializePlayer: false,
    writeAccountsSnapshot: true,
  });
};

export default deploy;
