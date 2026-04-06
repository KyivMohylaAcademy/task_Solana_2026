import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Marketplace } from "../target/types/marketplace";
import { expect } from "chai";

function getErrorMessage(error: any): string {
  if (error?.error?.errorMessage) return error.error.errorMessage;
  if (error?.message) return error.message;
  if (error?.logs) return error.logs.join(" ");
  return String(error);
}

describe("marketplace", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Marketplace as Program<Marketplace>;
  const admin = provider.wallet;

  let configPda: anchor.web3.PublicKey;
  const itemMint = anchor.web3.Keypair.generate();
  let listingPda: anchor.web3.PublicKey;

  const defaultPrices = [new anchor.BN(100), new anchor.BN(150), new anchor.BN(200), new anchor.BN(250)];

  before(async () => {
    [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    [listingPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("listing"), itemMint.publicKey.toBuffer()],
      program.programId
    );
  });

  describe("initialize", () => {
    it("should initialize marketplace config", async () => {
      try {
        const tx = await program.methods
          .initialize(defaultPrices)
          .accounts({
            config: configPda,
            admin: admin.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        console.log("Initialize transaction:", tx);
      } catch (e) {
        console.log("Config already initialized, verifying state...");
      }

      const config = await program.account.marketplaceConfig.fetch(configPda);
      expect(config.admin.toString()).to.equal(admin.publicKey.toString());
    });
  });

  describe("list_item", () => {
    it("should list a Cossack Saber for sale", async () => {
      const tx = await program.methods
        .listItem(0, null)
        .accounts({
          config: configPda,
          listing: listingPda,
          itemMint: itemMint.publicKey,
          seller: admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("List item transaction:", tx);

      const listing = await program.account.itemListing.fetch(listingPda);
      expect(listing.seller.toString()).to.equal(admin.publicKey.toString());
      expect(listing.itemMint.toString()).to.equal(itemMint.publicKey.toString());
      expect(listing.itemType).to.equal(0);
      expect(listing.price.toNumber()).to.equal(100);
      expect(listing.isActive).to.be.true;
    });

    it("should list with custom price", async () => {
      const customMint = anchor.web3.Keypair.generate();
      const [customListingPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), customMint.publicKey.toBuffer()],
        program.programId
      );

      const customPrice = new anchor.BN(500);

      const tx = await program.methods
        .listItem(1, customPrice)
        .accounts({
          config: configPda,
          listing: customListingPda,
          itemMint: customMint.publicKey,
          seller: admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("List item with custom price transaction:", tx);

      const listing = await program.account.itemListing.fetch(customListingPda);
      expect(listing.price.toNumber()).to.equal(500);
    });

    it("should fail with invalid item type", async () => {
      const mint = anchor.web3.Keypair.generate();
      const [listing] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), mint.publicKey.toBuffer()],
        program.programId
      );

      try {
        await program.methods
          .listItem(5, null)
          .accounts({
            config: configPda,
            listing: listing,
            itemMint: mint.publicKey,
            seller: admin.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Invalid item type");
      }
    });
  });

  describe("cancel_listing", () => {
    it("should cancel an active listing", async () => {
      const tx = await program.methods
        .cancelListing()
        .accounts({
          listing: listingPda,
          itemMint: itemMint.publicKey,
          seller: admin.publicKey,
        })
        .rpc();

      console.log("Cancel listing transaction:", tx);

      const listing = await program.account.itemListing.fetch(listingPda);
      expect(listing.isActive).to.be.false;
    });

    it("should fail to cancel inactive listing", async () => {
      try {
        await program.methods
          .cancelListing()
          .accounts({
            listing: listingPda,
            itemMint: itemMint.publicKey,
            seller: admin.publicKey,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Listing is not active");
      }
    });

    it("should fail when non-seller tries to cancel", async () => {
      // Create a fresh active listing for this test
      const freshMint = anchor.web3.Keypair.generate();
      const [freshListingPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), freshMint.publicKey.toBuffer()],
        program.programId
      );

      await program.methods
        .listItem(0, null)
        .accounts({
          config: configPda,
          listing: freshListingPda,
          itemMint: freshMint.publicKey,
          seller: admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const fakeSeller = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .cancelListing()
          .accounts({
            listing: freshListingPda,
            itemMint: freshMint.publicKey,
            seller: fakeSeller.publicKey,
          })
          .signers([fakeSeller])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Unauthorized");
      }
    });
  });

  describe("update_prices", () => {
    it("should update item prices", async () => {
      const newPrices = [new anchor.BN(120), new anchor.BN(180), new anchor.BN(240), new anchor.BN(300)];

      const tx = await program.methods
        .updatePrices(newPrices)
        .accounts({
          config: configPda,
          admin: admin.publicKey,
        })
        .rpc();

      console.log("Update prices transaction:", tx);

      const config = await program.account.marketplaceConfig.fetch(configPda);
      expect(config.itemPrices.map((p: any) => p.toNumber())).to.deep.equal([120, 180, 240, 300]);
    });

    it("should fail when non-admin tries to update prices", async () => {
      const fakeAdmin = anchor.web3.Keypair.generate();
      const prices = [new anchor.BN(100), new anchor.BN(100), new anchor.BN(100), new anchor.BN(100)];

      try {
        await program.methods
          .updatePrices(prices)
          .accounts({
            config: configPda,
            admin: fakeAdmin.publicKey,
          })
          .signers([fakeAdmin])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Unauthorized");
      }
    });
  });

  describe("sell_item_direct", () => {
    it("should validate item type", async () => {
      const [marketplaceAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("marketplace_authority")],
        program.programId
      );

      try {
        await program.methods
          .sellItemDirect(10)
          .accounts({
            config: configPda,
            marketplaceAuthority: marketplaceAuthority,
            itemMint: anchor.web3.Keypair.generate().publicKey,
            itemTokenAccount: anchor.web3.Keypair.generate().publicKey,
            magicTokenMint: anchor.web3.Keypair.generate().publicKey,
            sellerMagicTokenAccount: anchor.web3.Keypair.generate().publicKey,
            seller: admin.publicKey,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            token2022Program: anchor.utils.token.TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Invalid item type");
      }
    });
  });
});
