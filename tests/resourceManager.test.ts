import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ResourceManager } from "../target/types/resource_manager";
import { expect } from "chai";

describe("resource_manager", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.ResourceManager as Program<ResourceManager>;

  it("Initializes game config", async () => {
    const [gameConfigPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      program.programId
    );

    const tx = await program.methods
      .initializeConfig(provider.wallet.publicKey)
      .accounts({
        gameConfig: gameConfigPda,
        payer: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Initialize config tx:", tx);

    const gameConfig = await program.account.gameConfig.fetch(gameConfigPda);
    expect(gameConfig.admin.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(gameConfig.bump).to.be.a("number");
  });

  it("Should have proper error handling", async () => {
    // This would test error cases with invalid resource indices
    // Implementation would depend on mock setup
  });
});
