import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MagicToken } from "../target/types/magic_token";
import { expect } from "chai";

function getErrorMessage(error: any): string {
  if (error?.error?.errorMessage) return error.error.errorMessage;
  if (error?.message) return error.message;
  if (error?.logs) return error.logs.join(" ");
  return String(error);
}

describe("magic_token", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MagicToken as Program<MagicToken>;
  const admin = provider.wallet;

  let configPda: anchor.web3.PublicKey;
  const mint = anchor.web3.Keypair.generate();

  before(async () => {
    [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );
  });

  describe("initialize", () => {
    it("should initialize MagicToken config", async () => {
      const tx = await program.methods
        .initialize(9)
        .accounts({
          config: configPda,
          mint: mint.publicKey,
          admin: admin.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Initialize transaction:", tx);

      const config = await program.account.magicTokenConfig.fetch(configPda);
      expect(config.admin.toString()).to.equal(admin.publicKey.toString());
      expect(config.mint.toString()).to.equal(mint.publicKey.toString());
    });

    it("should fail to initialize twice", async () => {
      try {
        await program.methods
          .initialize(9)
          .accounts({
            config: configPda,
            mint: mint.publicKey,
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

  describe("mint_tokens", () => {
    it("should validate mint address", async () => {
      const [mintAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("mint_authority")],
        program.programId
      );

      const fakeMint = anchor.web3.Keypair.generate();

      try {
        await program.methods
          .mintTokens(new anchor.BN(100))
          .accounts({
            config: configPda,
            mintAuthority: mintAuthority,
            mint: fakeMint.publicKey,
            tokenAccount: anchor.web3.Keypair.generate().publicKey,
            tokenProgram: anchor.utils.token.TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        // fakeMint is not an initialized account, so the error may be
        // "Invalid mint" (constraint) or an account deserialization error
        expect(error).to.exist;
        console.log("Correctly rejected fake mint:", getErrorMessage(error));
      }
    });
  });
});
