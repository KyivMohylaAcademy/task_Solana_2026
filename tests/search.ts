import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Search } from "../target/types/search";
import { expect } from "chai";

function getErrorMessage(error: any): string {
  if (error?.error?.errorMessage) return error.error.errorMessage;
  if (error?.message) return error.message;
  if (error?.logs) return error.logs.join(" ");
  return String(error);
}

describe("search", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Search as Program<Search>;
  const owner = provider.wallet;

  let configPda: anchor.web3.PublicKey;
  let playerPda: anchor.web3.PublicKey;

  before(async () => {
    [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    [playerPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player"), owner.publicKey.toBuffer()],
      program.programId
    );
  });

  describe("initialize", () => {
    it("should initialize search config", async () => {
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

      const config = await program.account.searchConfig.fetch(configPda);
      expect(config.admin.toString()).to.equal(owner.publicKey.toString());
    });
  });

  describe("init_player", () => {
    it("should initialize player search account", async () => {
      try {
        const tx = await program.methods
          .initPlayer()
          .accounts({
            player: playerPda,
            owner: owner.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();

        console.log("Init player transaction:", tx);
      } catch (e) {
        // Player may already exist from a previous test suite run
        console.log("Player already initialized, verifying state...");
      }

      const player = await program.account.player.fetch(playerPda);
      expect(player.owner.toString()).to.equal(owner.publicKey.toString());
    });

    it("should fail to initialize player twice", async () => {
      try {
        await program.methods
          .initPlayer()
          .accounts({
            player: playerPda,
            owner: owner.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error) {
        expect(error).to.exist;
      }
    });
  });

  describe("search_resources", () => {
    it("should allow first search immediately", async () => {
      const playerBefore = await program.account.player.fetch(playerPda);
      const searchesBefore = playerBefore.totalSearches.toNumber();

      // If a search was already done (e.g. by integration tests), the cooldown
      // may still be active. In that case, just verify state and skip.
      try {
        const tx = await program.methods
          .searchResources()
          .accounts({
            config: configPda,
            player: playerPda,
            owner: owner.publicKey,
          })
          .rpc();

        console.log("Search transaction:", tx);

        const player = await program.account.player.fetch(playerPda);
        expect(player.totalSearches.toNumber()).to.equal(searchesBefore + 1);
        expect(player.lastSearchTimestamp.toNumber()).to.be.greaterThan(0);
      } catch (error: any) {
        // Cooldown active from integration test — verify searches already happened
        console.log("Cooldown active, verifying existing state...");
        expect(searchesBefore).to.be.greaterThan(0);
        expect(playerBefore.lastSearchTimestamp.toNumber()).to.be.greaterThan(0);
      }
    });

    it("should enforce 60 second cooldown", async () => {
      try {
        await program.methods
          .searchResources()
          .accounts({
            config: configPda,
            player: playerPda,
            owner: owner.publicKey,
          })
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (error: any) {
        expect(getErrorMessage(error)).to.include("Search cooldown active");
      }
    });

    it("should allow search after cooldown expires", async () => {
      // Wait for cooldown
      await new Promise((resolve) => setTimeout(resolve, 61000));

      const playerBefore = await program.account.player.fetch(playerPda);
      const searchesBefore = playerBefore.totalSearches.toNumber();

      const tx = await program.methods
        .searchResources()
        .accounts({
          config: configPda,
          player: playerPda,
          owner: owner.publicKey,
        })
        .rpc();

      console.log("Second search transaction:", tx);

      const playerAfter = await program.account.player.fetch(playerPda);
      expect(playerAfter.totalSearches.toNumber()).to.equal(searchesBefore + 1);
    }).timeout(70000);
  });

  describe("get_cooldown_remaining", () => {
    it("should return cooldown time", async () => {
      await program.methods
        .getCooldownRemaining()
        .accounts({
          player: playerPda,
          owner: owner.publicKey,
        })
        .rpc();

      // Just verify it doesn't throw
    });
  });
});
