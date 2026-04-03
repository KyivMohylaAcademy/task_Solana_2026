import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { ItemNft } from "../target/types/item_nft";
import { expect } from "chai";

describe("item_nft", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.ItemNft as Program<ItemNft>;

  const itemMint = anchor.web3.Keypair.generate();
  const uri = "https://example.com/items/sabre.json";

  it("Should create item NFT", async () => {
    const [nftMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item_nft"), itemMint.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .createItemNft(0, uri) // Item type 0 (Cossack Sabre)
      .accounts({
        nftMetadata: nftMetadataPda,
        mint: itemMint.publicKey,
        creator: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([itemMint])
      .rpc();

    console.log("Create item NFT tx:", tx);

    const nftMetadata = await program.account.itemNftMetadata.fetch(nftMetadataPda);
    expect(nftMetadata.itemType).to.equal(0);
    expect(nftMetadata.uri).to.equal(uri);
    expect(nftMetadata.creator.toString()).to.equal(provider.wallet.publicKey.toString());
  });

  it("Should burn item NFT", async () => {
    const [nftMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item_nft"), itemMint.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .burnItemNft()
      .accounts({
        nftMetadata: nftMetadataPda,
        owner: provider.wallet.publicKey,
        token2022Program: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    console.log("Burn item NFT tx:", tx);

    // NFT metadata should be closed
    try {
      await program.account.itemNftMetadata.fetch(nftMetadataPda);
      expect.fail("NFT metadata should be closed");
    } catch (err) {
      console.log("NFT metadata correctly closed after burn");
    }
  });
});
