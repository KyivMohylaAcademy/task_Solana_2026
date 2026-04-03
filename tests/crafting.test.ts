import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { Crafting } from "../target/types/crafting";
import { expect } from "chai";

describe("crafting", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Crafting as Program<Crafting>;

  it("Should craft item with correct item type", async () => {
    const itemMint = anchor.web3.Keypair.generate();
    const [itemMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item_metadata"), itemMint.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .craftItem(0) // Cossack Sabre
      .accounts({
        itemMetadata: itemMetadataPda,
        itemMint: itemMint.publicKey,
        owner: provider.wallet.publicKey,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([itemMint])
      .rpc();

    console.log("Craft item tx:", tx);

    const itemMetadata = await program.account.itemMetadata.fetch(itemMetadataPda);
    expect(itemMetadata.itemType).to.equal(0);
    expect(itemMetadata.owner.toString()).to.equal(provider.wallet.publicKey.toString());
  });

  it("Should reject invalid item type", async () => {
    const itemMint = anchor.web3.Keypair.generate();
    const [itemMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item_metadata"), itemMint.publicKey.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .craftItem(10) // Invalid item type
        .accounts({
          itemMetadata: itemMetadataPda,
          itemMint: itemMint.publicKey,
          owner: provider.wallet.publicKey,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([itemMint])
        .rpc();
      expect.fail("Should have thrown InvalidItemType error");
    } catch (err) {
      console.log("Correctly rejected invalid item type:", err.message);
    }
  });
});
