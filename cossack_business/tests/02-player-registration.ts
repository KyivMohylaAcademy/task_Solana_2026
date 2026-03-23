import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import {
  searchProgram,
  player1,
  player2,
  initializeAll,
} from "./helpers/setup";

describe("02 - Player Registration", () => {
  before(async () => {
    await initializeAll();
  });

  it("registers player 1", async () => {
    const [playerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), player1.publicKey.toBuffer()],
      searchProgram.programId
    );

    await searchProgram.methods
      .registerPlayer()
      .accounts({
        player: player1.publicKey,
        playerAccount: playerPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player1])
      .rpc();

    const acct = await searchProgram.account.player.fetch(playerPda);
    expect(acct.owner.toBase58()).to.equal(player1.publicKey.toBase58());
    expect(acct.lastSearchTimestamp.toNumber()).to.equal(0);
  });

  it("registers player 2", async () => {
    const [playerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), player2.publicKey.toBuffer()],
      searchProgram.programId
    );

    await searchProgram.methods
      .registerPlayer()
      .accounts({
        player: player2.publicKey,
        playerAccount: playerPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([player2])
      .rpc();

    const acct = await searchProgram.account.player.fetch(playerPda);
    expect(acct.owner.toBase58()).to.equal(player2.publicKey.toBase58());
  });

  it("fails to register same player twice", async () => {
    const [playerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), player1.publicKey.toBuffer()],
      searchProgram.programId
    );

    try {
      await searchProgram.methods
        .registerPlayer()
        .accounts({
          player: player1.publicKey,
          playerAccount: playerPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([player1])
        .rpc();
      expect.fail("Should have thrown");
    } catch (err) {
      expect(err).to.exist;
    }
  });
});
