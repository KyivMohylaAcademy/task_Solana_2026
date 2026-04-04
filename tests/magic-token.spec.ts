import { BN } from "@coral-xyz/anchor";
import { expectTxFailure } from "./utils/assert";
import { PROGRAM_IDS, ensureResourceAtas, getState } from "./utils/state";

describe("Magic token", () => {
  it("rejects direct mint without marketplace signer", async () => {
    const state = await getState();
    const { TOKEN_2022_PROGRAM_ID } = PROGRAM_IDS;

    const [sellerAta] = await ensureResourceAtas(state.provider, state.playerAuthority.publicKey, [state.magicMint]);

    await expectTxFailure(
      state.magicToken.methods
        .mintToSeller(new BN(5))
        .accountsStrict({
          config: state.magicConfig,
          marketplaceAuthority: state.marketAuthority,
          mint: state.magicMint,
          sellerAta,
          mintAuthority: state.magicMintAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          marketplaceProgram: state.marketplace.programId,
        })
        .rpc(),
      /Marketplace authority must sign/,
    );
  });
});
