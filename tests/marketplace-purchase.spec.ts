import { BN } from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

import {
  PROGRAM_IDS,
  airdrop,
  craftItem,
  ensureResourceAtas,
  farmResources,
  getState,
} from "./utils/state";
import { expectTxFailure } from "./utils/assert";

describe("Marketplace purchase", () => {
  before(async () => {
    const state = await getState();
    await farmResources(state, Array(6).fill(5));
  });

  it("fails when buyer lacks MagicToken balance", async () => {
    const state = await getState();
    const { TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } = PROGRAM_IDS;

    const buyer = Keypair.generate();
    await airdrop(state.provider.connection, buyer.publicKey);

    const [buyerMagicAta] = await ensureResourceAtas(state.provider, buyer.publicKey, [state.magicMint]);
    const [sellerMagicAta] = await ensureResourceAtas(state.provider, state.playerAuthority.publicKey, [state.magicMint]);

    const item = await craftItem(state, 0);
    const listingPda = PublicKey.findProgramAddressSync(
      [Buffer.from("listing"), item.mint.publicKey.toBuffer()],
      state.marketplace.programId,
    )[0];
    const escrowAta = getAssociatedTokenAddressSync(
      item.mint.publicKey,
      state.marketAuthority,
      true,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    await state.marketplace.methods
      .list(new BN(7))
      .accountsStrict({
        seller: state.playerAuthority.publicKey,
        itemMint: item.mint.publicKey,
        sellerItemAta: item.ata,
        escrowItemAta: escrowAta,
        listing: listingPda,
        marketAuthority: state.marketAuthority,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([state.playerAuthority])
      .rpc();

    await expectTxFailure(
      state.marketplace.methods
        .purchase()
        .accountsStrict({
          buyer: buyer.publicKey,
          seller: state.playerAuthority.publicKey,
          itemMint: item.mint.publicKey,
          escrowItemAta: escrowAta,
          buyerMagicAta,
          sellerMagicAta,
          listing: listingPda,
          config: state.magicConfig,
          magicMintAuthority: state.magicMintAuthority,
          magicMint: state.magicMint,
          marketAuthority: state.marketAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          magicTokenProgram: state.magicToken.programId,
          marketplaceProgram: state.marketplace.programId,
        })
        .signers([buyer])
        .rpc(),
      /insufficient funds|custom program error: 0x1|Program .* failed|Program failed to complete/i,
    );
  });
});
