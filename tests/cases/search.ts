import { expect } from "chai";
import {
  ComputeBudgetProgram,
  Keypair,
  SystemProgram,
} from "@solana/web3.js";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import type { SuiteContext } from "../support/context";

export function registerSearchTests(ctx: SuiteContext) {
  const {
    searchProgram,
    resourceMints,
    resourceTokenAccounts,
    playerAddress,
    searchAccounts,
    expectReject,
    expectAllAccountsExist,
    waitForBalances,
    fundUser,
  } = ctx;

  it("enforces the 60-second search cooldown and mints exactly three resources", async function () {
    this.timeout(60_000);

    const explorer = Keypair.generate();
    await fundUser(explorer, 0.4);

    const [playerPda] = playerAddress(explorer.publicKey);
    const explorerResourceAccounts = resourceTokenAccounts(explorer.publicKey);

    await searchProgram.methods
      .initializePlayer()
      .accounts({
        owner: explorer.publicKey,
        player: playerPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([explorer])
      .rpc();

    await searchProgram.methods
      .searchResources()
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 500_000 }),
      ])
      .accounts(searchAccounts(explorer.publicKey, playerPda))
      .signers([explorer])
      .rpc();

    const balances = await waitForBalances(
      explorerResourceAccounts,
      (items) => items.reduce((sum, value) => sum + value, 0) === 3,
    );
    const totalMinted = balances.reduce((sum, value) => sum + value, 0);
    expect(totalMinted).to.equal(3);
    await expectAllAccountsExist(explorerResourceAccounts, TOKEN_2022_PROGRAM_ID);

    await expectReject(
      searchProgram.methods
        .searchResources()
        .accounts(searchAccounts(explorer.publicKey, playerPda))
        .signers([explorer])
        .rpc(),
    );

    const playerState = await searchProgram.account.player.fetch(playerPda);
    expect(Number(playerState.lastSearchTimestamp)).to.be.greaterThan(0);
  });

  it("rejects search account layouts with wrong resource mints", async function () {
    this.timeout(60_000);

    const explorer = Keypair.generate();
    await fundUser(explorer, 0.4);

    const [playerPda] = playerAddress(explorer.publicKey);
    await searchProgram.methods
      .initializePlayer()
      .accounts({
        owner: explorer.publicKey,
        player: playerPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([explorer])
      .rpc();

    const accounts = searchAccounts(explorer.publicKey, playerPda);
    await expectReject(
      searchProgram.methods
        .searchResources()
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 500_000 }),
        ])
        .accounts({
          ...accounts,
          woodMint: resourceMints[1],
          woodAccount: resourceTokenAccounts(explorer.publicKey)[1],
          ironMint: resourceMints[0],
          ironAccount: resourceTokenAccounts(explorer.publicKey)[0],
        })
        .signers([explorer])
        .rpc(),
    );
  });
}
