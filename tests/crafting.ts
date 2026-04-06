import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Crafting } from "../target/types/crafting";
import { expect } from "chai";

function getErrorMessage(error: any): string {
  if (error?.error?.errorMessage) return error.error.errorMessage;
  if (error?.message) return error.message;
  if (error?.logs) return error.logs.join(" ");
  return String(error);
}

describe("crafting", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Crafting as Program<Crafting>;
  const owner = provider.wallet;

  let configPda: anchor.web3.PublicKey;

  before(async () => {
    [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );
  });

  describe("initialize", () => {
    it("should initialize crafting config", async () => {
      const tx = await program.methods
        .initialize()
        .accounts({
          config: configPda,
          admin: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Initialize transaction:", tx);

      const config = await program.account.craftingConfig.fetch(configPda);
      expect(config.admin.toString()).to.equal(owner.publicKey.toString());
      expect(config.totalCrafted.toNumber()).to.equal(0);
    });
  });

  describe("craft_item", () => {
    it("should craft a Cossack Saber (item type 0)", async () => {
      const tx = await program.methods
        .craftItem(0)
        .accounts({
          config: configPda,
          owner: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Craft Cossack Saber transaction:", tx);

      const config = await program.account.craftingConfig.fetch(configPda);
      expect(config.totalCrafted.toNumber()).to.equal(1);
    });

    it("should craft an Elder Staff (item type 1)", async () => {
      const tx = await program.methods
        .craftItem(1)
        .accounts({
          config: configPda,
          owner: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Craft Elder Staff transaction:", tx);

      const config = await program.account.craftingConfig.fetch(configPda);
      expect(config.totalCrafted.toNumber()).to.equal(2);
    });

    it("should craft Characternik Armor (item type 2)", async () => {
      const tx = await program.methods
        .craftItem(2)
        .accounts({
          config: configPda,
          owner: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Craft Characternik Armor transaction:", tx);

      const config = await program.account.craftingConfig.fetch(configPda);
      expect(config.totalCrafted.toNumber()).to.equal(3);
    });

    it("should craft Battle Bracelet (item type 3)", async () => {
      const tx = await program.methods
        .craftItem(3)
        .accounts({
          config: configPda,
          owner: owner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Craft Battle Bracelet transaction:", tx);

      const config = await program.account.craftingConfig.fetch(configPda);
      expect(config.totalCrafted.toNumber()).to.equal(4);
    });

    it("should fail with invalid item type", async () => {
      try {
        await program.methods
          .craftItem(4)
          .accounts({
            config: configPda,
            owner: owner.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Invalid item type");
      }
    });
  });

  describe("burn_resource", () => {
    it("should validate resource ID", async () => {
      try {
        await program.methods
          .burnResource(6, new anchor.BN(1))
          .accounts({
            mint: anchor.web3.Keypair.generate().publicKey,
            tokenAccount: anchor.web3.Keypair.generate().publicKey,
            owner: owner.publicKey,
            tokenProgram: anchor.utils.token.TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Invalid resource ID");
      }
    });

    it("should validate amount is greater than zero", async () => {
      try {
        await program.methods
          .burnResource(0, new anchor.BN(0))
          .accounts({
            mint: anchor.web3.Keypair.generate().publicKey,
            tokenAccount: anchor.web3.Keypair.generate().publicKey,
            owner: owner.publicKey,
            tokenProgram: anchor.utils.token.TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Invalid amount");
      }
    });
  });
});
