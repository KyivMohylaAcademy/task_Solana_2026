import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { Marketplace } from "../target/types/marketplace";
import { expect } from "chai";
import { BN } from "bn.js";

describe("marketplace", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Marketplace as Program<Marketplace>;

  const itemMint = anchor.web3.Keypair.generate();
  const price = new BN(1000);

  it("Should list item for sale", async () => {
    const [listingPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("listing"), itemMint.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .listItem(price)
      .accounts({
        listing: listingPda,
        itemMint: itemMint.publicKey,
        seller: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([itemMint])
      .rpc();

    console.log("List item tx:", tx);

    const listing = await program.account.listing.fetch(listingPda);
    expect(listing.seller.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(listing.price.toNumber()).to.equal(price.toNumber());
  });

  it("Should buy item from marketplace", async () => {
    const [listingPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("listing"), itemMint.publicKey.toBuffer()],
      program.programId
    );

    const buyer = anchor.web3.Keypair.generate();

    const tx = await program.methods
      .buyItem()
      .accounts({
        listing: listingPda,
        itemMint: itemMint.publicKey,
        buyer: buyer.publicKey,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([buyer, itemMint])
      .rpc();

    console.log("Buy item tx:", tx);

    // Listing should be closed after purchase
    try {
      await program.account.listing.fetch(listingPda);
      expect.fail("Listing should be closed");
    } catch (err) {
      console.log("Listing correctly closed after purchase");
    }
  });
});
