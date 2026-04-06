import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ItemNft } from "../target/types/item_nft";
import { expect } from "chai";

function getErrorMessage(error: any): string {
  if (error?.error?.errorMessage) return error.error.errorMessage;
  if (error?.message) return error.message;
  if (error?.logs) return error.logs.join(" ");
  return String(error);
}

describe("item_nft", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ItemNft as Program<ItemNft>;
  const owner = provider.wallet;

  let configPda: anchor.web3.PublicKey;
  const itemMint = anchor.web3.Keypair.generate();
  let itemMetadataPda: anchor.web3.PublicKey;
  let transferredOwner: anchor.web3.Keypair;

  before(async () => {
    [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    [itemMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item"), itemMint.publicKey.toBuffer()],
      program.programId
    );
  });

  describe("initialize", () => {
    it("should initialize item NFT config", async () => {
      try {
        const tx = await program.methods
          .initialize()
          .accounts({
            config: configPda,
            admin: owner.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        console.log("Initialize transaction:", tx);
      } catch (e) {
        console.log("Config already initialized, verifying state...");
      }

      const config = await program.account.itemConfig.fetch(configPda);
      expect(config.admin.toString()).to.equal(owner.publicKey.toString());
    });
  });

  describe("create_item", () => {
    it("should create a Cossack Saber NFT (type 0)", async () => {
      const configBefore = await program.account.itemConfig.fetch(configPda);
      const mintedBefore = configBefore.totalItemsMinted.toNumber();

      const tx = await program.methods
        .createItem(0, "https://example.com/metadata/saber.json")
        .accounts({
          config: configPda,
          itemMetadata: itemMetadataPda,
          mint: itemMint.publicKey,
          tokenAccount: anchor.web3.Keypair.generate().publicKey,
          owner: owner.publicKey,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();

      console.log("Create item transaction:", tx);

      const itemMetadata = await program.account.itemMetadata.fetch(itemMetadataPda);
      expect(itemMetadata.itemType).to.equal(0);
      expect(itemMetadata.owner.toString()).to.equal(owner.publicKey.toString());
      expect(itemMetadata.mint.toString()).to.equal(itemMint.publicKey.toString());

      const config = await program.account.itemConfig.fetch(configPda);
      expect(config.totalItemsMinted.toNumber()).to.equal(mintedBefore + 1);
    });

    it("should fail with invalid item type", async () => {
      const mint = anchor.web3.Keypair.generate();
      const [metadata] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("item"), mint.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .createItem(5, "https://example.com/metadata/invalid.json")
          .accounts({
            config: configPda,
            itemMetadata: metadata,
            mint: mint.publicKey,
            tokenAccount: anchor.web3.Keypair.generate().publicKey,
            owner: owner.publicKey,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Invalid item type");
      }
    });
  });

  describe("transfer_item", () => {
    it("should transfer item ownership", async () => {
      const newOwner = anchor.web3.Keypair.generate();
      transferredOwner = newOwner;

      const tx = await program.methods
        .transferItem()
        .accounts({
          itemMetadata: itemMetadataPda,
          mint: itemMint.publicKey,
          owner: owner.publicKey,
          newOwner: newOwner.publicKey,
        })
        .rpc();

      console.log("Transfer item transaction:", tx);

      const itemMetadata = await program.account.itemMetadata.fetch(itemMetadataPda);
      expect(itemMetadata.owner.toString()).to.equal(newOwner.publicKey.toString());
    });

    it("should fail transfer from non-owner", async () => {
      const fakeOwner = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .transferItem()
          .accounts({
            itemMetadata: itemMetadataPda,
            mint: itemMint.publicKey,
            owner: fakeOwner.publicKey,
            newOwner: anchor.web3.Keypair.generate().publicKey,
          })
          .signers([fakeOwner])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Unauthorized");
      }
    });
  });

  describe("burn_item", () => {
    it("should burn an item NFT", async () => {
      // First transfer back to original owner (current owner is transferredOwner)
      const tx1 = await program.methods
        .transferItem()
        .accounts({
          itemMetadata: itemMetadataPda,
          mint: itemMint.publicKey,
          owner: transferredOwner.publicKey,
          newOwner: owner.publicKey,
        })
        .signers([transferredOwner])
        .rpc();

      // Then burn
      const tx2 = await program.methods
        .burnItem()
        .accounts({
          itemMetadata: itemMetadataPda,
          mint: itemMint.publicKey,
          tokenAccount: anchor.web3.Keypair.generate().publicKey,
          owner: owner.publicKey,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .rpc();

      console.log("Burn item transaction:", tx2);

      // Metadata account should be closed
      try {
        await program.account.itemMetadata.fetch(itemMetadataPda);
        expect.fail("Account should be closed");
      } catch (error) {
        expect(error.message).to.include("Account does not exist");
      }
    });
  });
});
