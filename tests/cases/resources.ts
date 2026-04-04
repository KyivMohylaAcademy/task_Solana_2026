import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";
import { expect } from "chai";
import {
  TOKEN_2022_PROGRAM_ID,
  createBurnInstruction,
} from "@solana/spl-token";
import {
  ComputeBudgetProgram,
  Transaction,
  SystemProgram,
} from "@solana/web3.js";
import type { SuiteContext } from "../support/context";

export function registerResourceTests(ctx: SuiteContext) {
  const {
    provider,
    resourceManager,
    searchProgram,
    gameConfig,
    resourceAuthority,
    resourceMints,
    invalidResourceId,
    playerAddress,
    resourceTokenAccounts,
    searchAccounts,
    ensureAllResourceAtas,
    waitForBalances,
    expectReject,
    fundUser,
  } = ctx;

  it("blocks direct resource burn through Token-2022", async function () {
    this.timeout(60_000);

    const miner = anchor.web3.Keypair.generate();
    await fundUser(miner, 0.4);

    const [playerPda] = playerAddress(miner.publicKey);
    const minerResourceAccounts = resourceTokenAccounts(miner.publicKey);
    await ensureAllResourceAtas(miner.publicKey);

    await searchProgram.methods
      .initializePlayer()
      .accounts({
        owner: miner.publicKey,
        player: playerPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([miner])
      .rpc();

    await searchProgram.methods
      .searchResources()
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 500_000 }),
      ])
      .accounts(searchAccounts(miner.publicKey, playerPda))
      .signers([miner])
      .rpc();

    const balances = await waitForBalances(
      minerResourceAccounts,
      (items) => items.reduce((sum, value) => sum + value, 0) === 3,
    );
    const burnIndex = balances.findIndex((value) => value > 0);
    expect(burnIndex).to.not.equal(-1);

    await expectReject(
      provider.sendAndConfirm(
        new Transaction().add(
          createBurnInstruction(
            minerResourceAccounts[burnIndex],
            resourceMints[burnIndex],
            miner.publicKey,
            1,
            [],
            TOKEN_2022_PROGRAM_ID,
          ),
        ),
        [miner],
      ),
    );
  });

  it("rejects direct resource burn and transfer guard branches", async function () {
    this.timeout(90_000);

    const owner = anchor.web3.Keypair.generate();
    const outsider = anchor.web3.Keypair.generate();
    const recipient = anchor.web3.Keypair.generate();
    await fundUser(owner, 0.4);
    await fundUser(outsider, 0.2);
    await fundUser(recipient, 0.2);

    const [playerPda] = playerAddress(owner.publicKey);
    const ownerResourceAccounts = resourceTokenAccounts(owner.publicKey);
    const recipientResourceAccounts = resourceTokenAccounts(recipient.publicKey);
    await ensureAllResourceAtas(recipient.publicKey);

    await searchProgram.methods
      .initializePlayer()
      .accounts({
        owner: owner.publicKey,
        player: playerPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([owner])
      .rpc();

    await searchProgram.methods
      .searchResources()
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 500_000 }),
      ])
      .accounts(searchAccounts(owner.publicKey, playerPda))
      .signers([owner])
      .rpc();

    const balances = await waitForBalances(
      ownerResourceAccounts,
      (items) => items.reduce((sum, value) => sum + value, 0) === 3,
    );
    const sourceIndex = balances.findIndex((value) => value > 0);
    expect(sourceIndex).to.not.equal(-1);
    const wrongIndex = (sourceIndex + 1) % resourceMints.length;

    await expectReject(
      resourceManager.methods
        .burnResource(wrongIndex, new BN(1))
        .accounts({
          authority: owner.publicKey,
          owner: owner.publicKey,
          gameConfig,
          mint: resourceMints[wrongIndex],
          source: ownerResourceAccounts[sourceIndex],
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([owner])
        .rpc(),
    );

    await expectReject(
      resourceManager.methods
        .burnResource(sourceIndex, new BN(1))
        .accounts({
          authority: outsider.publicKey,
          owner: owner.publicKey,
          gameConfig,
          mint: resourceMints[sourceIndex],
          source: ownerResourceAccounts[sourceIndex],
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([owner, outsider])
        .rpc(),
    );

    await expectReject(
      resourceManager.methods
        .transferResource(invalidResourceId, new BN(1))
        .accounts({
          owner: owner.publicKey,
          gameConfig,
          mint: resourceMints[sourceIndex],
          source: ownerResourceAccounts[sourceIndex],
          destination: recipientResourceAccounts[sourceIndex],
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([owner])
        .rpc(),
    );

    await expectReject(
      resourceManager.methods
        .transferResource(sourceIndex, new BN(1))
        .accounts({
          owner: owner.publicKey,
          gameConfig,
          mint: resourceMints[wrongIndex],
          source: ownerResourceAccounts[sourceIndex],
          destination: recipientResourceAccounts[sourceIndex],
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([owner])
        .rpc(),
    );

    await expectReject(
      resourceManager.methods
        .transferResource(sourceIndex, new BN(1))
        .accounts({
          owner: outsider.publicKey,
          gameConfig,
          mint: resourceMints[sourceIndex],
          source: ownerResourceAccounts[sourceIndex],
          destination: recipientResourceAccounts[sourceIndex],
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([outsider])
        .rpc(),
    );
  });

  it("blocks same-account resource transfers", async function () {
    this.timeout(60_000);

    const collector = anchor.web3.Keypair.generate();
    await fundUser(collector, 0.4);

    const [playerPda] = playerAddress(collector.publicKey);
    const collectorResourceAccounts = resourceTokenAccounts(collector.publicKey);
    await ensureAllResourceAtas(collector.publicKey);

    await searchProgram.methods
      .initializePlayer()
      .accounts({
        owner: collector.publicKey,
        player: playerPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([collector])
      .rpc();

    await searchProgram.methods
      .searchResources()
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 500_000 }),
      ])
      .accounts(searchAccounts(collector.publicKey, playerPda))
      .signers([collector])
      .rpc();

    const balances = await waitForBalances(
      collectorResourceAccounts,
      (items) => items.reduce((sum, value) => sum + value, 0) === 3,
    );
    const transferIndex = balances.findIndex((value) => value > 0);
    expect(transferIndex).to.not.equal(-1);

    await expectReject(
      resourceManager.methods
        .transferResource(transferIndex, new BN(1))
        .accounts({
          owner: collector.publicKey,
          gameConfig,
          mint: resourceMints[transferIndex],
          source: collectorResourceAccounts[transferIndex],
          destination: collectorResourceAccounts[transferIndex],
          mintAuthority: resourceAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([collector])
        .rpc(),
    );
  });
}
