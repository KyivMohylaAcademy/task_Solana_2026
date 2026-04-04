import { BN } from "@coral-xyz/anchor";
import { expect } from "chai";
import { getAccount, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { PublicKey, SystemProgram } from "@solana/web3.js";

import { PROGRAM_IDS, craftItem, farmResources, getState } from "./utils/state";
import { expectTxFailure } from "./utils/assert";

describe("Marketplace", () => {
  before(async () => {
    const state = await getState();
    await farmResources(state, Array(6).fill(5));
  });

  it("lists and delists an item", async () => {
    const state = await getState();
    const { TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } = PROGRAM_IDS;

    const item = await craftItem(state, 0);
    const listingPda = getListingPda(state.marketplace.programId, item.mint.publicKey);
    const escrowAta = getAssociatedTokenAddressSync(
      item.mint.publicKey,
      state.marketAuthority,
      true,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    const price = new BN(10);

    await state.marketplace.methods
      .list(price)
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

    const listing = await state.marketplace.account.listing.fetch(listingPda);
    expect(listing.seller.toBase58()).to.equal(state.playerAuthority.publicKey.toBase58());
    expect(listing.itemMint.toBase58()).to.equal(item.mint.publicKey.toBase58());
    expect(listing.price.toNumber()).to.equal(price.toNumber());

    const escrowAccount = await getAccount(state.provider.connection, escrowAta, undefined, TOKEN_2022_PROGRAM_ID);
    expect(Number(escrowAccount.amount)).to.equal(1);

    await state.marketplace.methods
      .delist()
      .accountsStrict({
        seller: state.playerAuthority.publicKey,
        itemMint: item.mint.publicKey,
        sellerItemAta: item.ata,
        escrowItemAta: escrowAta,
        listing: listingPda,
        marketAuthority: state.marketAuthority,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([state.playerAuthority])
      .rpc();

    await expectTxFailure(
      state.marketplace.account.listing.fetch(listingPda),
      /Account does not exist|Could not find|not found/i,
    );

    const sellerAccount = await getAccount(state.provider.connection, item.ata, undefined, TOKEN_2022_PROGRAM_ID);
    expect(Number(sellerAccount.amount)).to.equal(1);
  });
});

function getListingPda(programId: PublicKey, mint: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("listing"), mint.toBuffer()],
    programId,
  )[0];
}
