import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import {
  provider,
  rmProgram,
  searchProgram,
  player1,
  player2,
  resourceMints,
  searchCallerAuth,
  searchCooldown,
  initializeAll,
} from "./helpers/setup";
import {
  TOKEN_2022_PROGRAM_ID,
  createResourceAtas,
} from "./helpers/utils";

describe("03 - Search", () => {
  let gameConfigPda: PublicKey;
  let mintAuthorityPda: PublicKey;
  let player1Atas: PublicKey[];

  before(async () => {
    await initializeAll();
    const setup = require("./helpers/setup");
    gameConfigPda = setup.gameConfigPda;
    mintAuthorityPda = setup.mintAuthorityPda;
    player1Atas = await createResourceAtas(player1.publicKey);
  });

  it("player searches and receives 3 resources", async () => {
    const [playerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), player1.publicKey.toBuffer()],
      searchProgram.programId
    );

    const remainingAccounts = [
      ...resourceMints.map((m) => ({
        pubkey: m.publicKey, isSigner: false, isWritable: true,
      })),
      ...player1Atas.map((a) => ({
        pubkey: a, isSigner: false, isWritable: true,
      })),
    ];

    await searchProgram.methods
      .searchResources()
      .accounts({
        player: player1.publicKey,
        playerAccount: playerPda,
        callerAuthority: searchCallerAuth,
        gameConfig: gameConfigPda,
        mintAuthority: mintAuthorityPda,
        resourceManagerProgram: rmProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts(remainingAccounts)
      .signers([player1])
      .rpc();

    let totalResources = 0;
    for (const ata of player1Atas) {
      const info = await provider.connection.getTokenAccountBalance(ata);
      totalResources += parseInt(info.value.amount);
    }
    expect(totalResources).to.equal(3);
  });

  it("fails if searched within cooldown", async () => {
    const [playerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), player1.publicKey.toBuffer()],
      searchProgram.programId
    );

    const remainingAccounts = [
      ...resourceMints.map((m) => ({
        pubkey: m.publicKey, isSigner: false, isWritable: true,
      })),
      ...player1Atas.map((a) => ({
        pubkey: a, isSigner: false, isWritable: true,
      })),
    ];

    try {
      await searchProgram.methods
        .searchResources()
        .accounts({
          player: player1.publicKey,
          playerAccount: playerPda,
          callerAuthority: searchCallerAuth,
          gameConfig: gameConfigPda,
          mintAuthority: mintAuthorityPda,
          resourceManagerProgram: rmProgram.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(remainingAccounts)
        .signers([player1])
        .rpc();
      expect.fail("Should have thrown - cooldown not expired");
    } catch (err) {
      expect(err.toString()).to.include("SearchCooldown");
    }
  });

  it("succeeds after cooldown expires", async () => {
    await new Promise((resolve) => setTimeout(resolve, (searchCooldown + 1) * 1000));

    const [playerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), player1.publicKey.toBuffer()],
      searchProgram.programId
    );

    const remainingAccounts = [
      ...resourceMints.map((m) => ({
        pubkey: m.publicKey, isSigner: false, isWritable: true,
      })),
      ...player1Atas.map((a) => ({
        pubkey: a, isSigner: false, isWritable: true,
      })),
    ];

    await searchProgram.methods
      .searchResources()
      .accounts({
        player: player1.publicKey,
        playerAccount: playerPda,
        callerAuthority: searchCallerAuth,
        gameConfig: gameConfigPda,
        mintAuthority: mintAuthorityPda,
        resourceManagerProgram: rmProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts(remainingAccounts)
      .signers([player1])
      .rpc();

    let totalResources = 0;
    for (const ata of player1Atas) {
      const info = await provider.connection.getTokenAccountBalance(ata);
      totalResources += parseInt(info.value.amount);
    }
    expect(totalResources).to.equal(6);
  });

  it("different players can search independently", async () => {
    const player2Atas = await createResourceAtas(player2.publicKey);
    const [playerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), player2.publicKey.toBuffer()],
      searchProgram.programId
    );

    const remainingAccounts = [
      ...resourceMints.map((m) => ({
        pubkey: m.publicKey, isSigner: false, isWritable: true,
      })),
      ...player2Atas.map((a) => ({
        pubkey: a, isSigner: false, isWritable: true,
      })),
    ];

    await searchProgram.methods
      .searchResources()
      .accounts({
        player: player2.publicKey,
        playerAccount: playerPda,
        callerAuthority: searchCallerAuth,
        gameConfig: gameConfigPda,
        mintAuthority: mintAuthorityPda,
        resourceManagerProgram: rmProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts(remainingAccounts)
      .signers([player2])
      .rpc();

    let total = 0;
    for (const ata of player2Atas) {
      const info = await provider.connection.getTokenAccountBalance(ata);
      total += parseInt(info.value.amount);
    }
    expect(total).to.equal(3);
  });
});
