import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ResourceManager } from "../target/types/resource_manager";
import { expect } from "chai";

function getErrorMessage(error: any): string {
  if (error?.error?.errorMessage) return error.error.errorMessage;
  if (error?.message) return error.message;
  if (error?.logs) return error.logs.join(" ");
  return String(error);
}

describe("resource_manager", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ResourceManager as Program<ResourceManager>;
  const admin = provider.wallet;

  let configPda: anchor.web3.PublicKey;
  let configBump: number;

  before(async () => {
    [configPda, configBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );
  });

  describe("initialize", () => {
    it("should initialize resource manager config", async () => {
      const tx = await program.methods
        .initialize()
        .accounts({
          config: configPda,
          admin: admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Initialize transaction:", tx);

      const config = await program.account.resourceConfig.fetch(configPda);
      expect(config.admin.toString()).to.equal(admin.publicKey.toString());
      expect(config.bump).to.equal(configBump);
    });

    it("should fail to initialize twice", async () => {
      try {
        await program.methods
          .initialize()
          .accounts({
            config: configPda,
            admin: admin.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("create_resource_mint", () => {
    const resourceMints: anchor.web3.Keypair[] = [];
    const resourceNames = ["Wood", "Iron", "Gold", "Leather", "Stone", "Diamond"];
    const resourceSymbols = ["WOOD", "IRON", "GOLD", "LEATHER", "STONE", "DIAMOND"];

    it("should create all 6 resource mints", async () => {
      for (let i = 0; i < 6; i++) {
        const mint = anchor.web3.Keypair.generate();
        resourceMints.push(mint);

        await program.methods
          .createResourceMint(i, resourceNames[i], resourceSymbols[i])
          .accounts({
            config: configPda,
            mint: mint.publicKey,
            admin: admin.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: anchor.utils.token.TOKEN_2022_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .rpc();

        const config = await program.account.resourceConfig.fetch(configPda);
        expect(config.resourceMints[i].toString()).to.equal(mint.publicKey.toString());
      }
    });

    it("should fail with invalid resource ID", async () => {
      const mint = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .createResourceMint(6, "Invalid", "INV")
          .accounts({
            config: configPda,
            mint: mint.publicKey,
            admin: admin.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: anchor.utils.token.TOKEN_2022_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Invalid resource ID");
      }
    });
  });

  describe("mint_resource", () => {
    it("should validate mint address", async () => {
      const fakeMint = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .mintResource(0, new anchor.BN(100))
          .accounts({
            config: configPda,
            mint: fakeMint.publicKey,
            tokenAccount: anchor.web3.Keypair.generate().publicKey,
            authority: anchor.web3.Keypair.generate().publicKey,
            tokenProgram: anchor.utils.token.TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Invalid mint");
      }
    });
  });

  describe("burn_resource", () => {
    it("should validate resource ID for burning", async () => {
      try {
        await program.methods
          .burnResource(10, new anchor.BN(1))
          .accounts({
            config: configPda,
            mint: anchor.web3.Keypair.generate().publicKey,
            tokenAccount: anchor.web3.Keypair.generate().publicKey,
            owner: admin.publicKey,
            tokenProgram: anchor.utils.token.TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Invalid resource ID");
      }
    });
  });
});
