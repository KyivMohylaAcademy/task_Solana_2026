// uses bankrun for clock control — state is not shared with the other test files
import * as anchor from "@coral-xyz/anchor";
import { BankrunProvider, startAnchor } from "anchor-bankrun";
import { Clock } from "solana-bankrun";
import { ComputeBudgetProgram, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { assert } from "chai";
import { ResourceManager } from "../../../target/types/resource_manager";
import { MagicToken } from "../../../target/types/magic_token";
import { Search } from "../../../target/types/search";

const RESOURCE_NAMES = ["Wood", "Iron", "Gold", "Leather", "Stone", "Diamond"];
const RESOURCE_SYMBOLS = ["WOOD", "IRON", "GOLD", "LTHR", "STON", "DIAM"];
const RESOURCE_URIS = Array.from({ length: 6 }, (_, i) => `https://REPLACE_ME/resource${i}.json`);

describe("search (bankrun)", () => {
  let context: Awaited<ReturnType<typeof startAnchor>>;
  let provider: BankrunProvider;
  let rmProgram: anchor.Program<ResourceManager>;
  let mtProgram: anchor.Program<MagicToken>;
  let searchProgram: anchor.Program<Search>;

  let resourceMints: Keypair[];
  let magicTokenMintKp: Keypair;
  let gameConfigPda: PublicKey;
  let playerPda: PublicKey;

  before(async () => {
    // Start bankrun with all program artifacts from the build.
    context = await startAnchor(".", [], []);
    provider = new BankrunProvider(context);
    anchor.setProvider(provider);

    rmProgram = anchor.workspace.ResourceManager as anchor.Program<ResourceManager>;
    mtProgram = anchor.workspace.MagicToken as anchor.Program<MagicToken>;
    searchProgram = anchor.workspace.Search as anchor.Program<Search>;

    resourceMints = Array.from({ length: 6 }, () => Keypair.generate());
    magicTokenMintKp = Keypair.generate();

    const [resourceMintAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("resource_mint_auth")],
      rmProgram.programId
    );
    const [magicMintAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("magic_mint_auth")],
      mtProgram.programId
    );
    [gameConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_config")],
      rmProgram.programId
    );
    [playerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), provider.wallet.publicKey.toBuffer()],
      searchProgram.programId
    );

    for (let i = 0; i < 6; i++) {
      await rmProgram.methods
        .initResourceMint(i, RESOURCE_NAMES[i], RESOURCE_SYMBOLS[i], RESOURCE_URIS[i])
        .accounts({
          payer: provider.wallet.publicKey,
          mint: resourceMints[i].publicKey,
          resourceMintAuth,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([resourceMints[i]])
        .rpc();
    }

    await mtProgram.methods
      .initMagicTokenMint("MagicToken", "MGT", "https://REPLACE_ME/magic-token.json")
      .accounts({
        payer: provider.wallet.publicKey,
        mint: magicTokenMintKp.publicKey,
        magicMintAuth,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([magicTokenMintKp])
      .rpc();

    await rmProgram.methods
      .initialize(
        resourceMints.map((kp) => kp.publicKey),
        magicTokenMintKp.publicKey,
        [new anchor.BN(10), new anchor.BN(15), new anchor.BN(20), new anchor.BN(25)]
      )
      .accounts({
        admin: provider.wallet.publicKey,
        gameConfig: gameConfigPda,
        systemProgram: SystemProgram.programId,
      })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
      .rpc();
  });

  it("registers a player", async () => {
    await searchProgram.methods
      .registerPlayer()
      .accounts({
        signer: provider.wallet.publicKey,
        player: playerPda,
        systemProgram: SystemProgram.programId,
      })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
      .rpc();

    const playerAccount = await searchProgram.account.player.fetch(playerPda);
    assert.equal(
      playerAccount.owner.toBase58(),
      provider.wallet.publicKey.toBase58()
    );
    assert.equal(playerAccount.lastSearchTimestamp.toNumber(), 0);
  });

  it("searches once and receives 3 resource tokens", async () => {
    const [resourceMintAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("resource_mint_auth")],
      rmProgram.programId
    );
    const [cpiAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("cpi_auth")],
      searchProgram.programId
    );

    const playerAtas = resourceMints.map((kp) =>
      getAssociatedTokenAddressSync(kp.publicKey, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID)
    );

    await searchProgram.methods
      .searchResources()
      .accounts({
        playerWallet: provider.wallet.publicKey,
        player: playerPda,
        owner: provider.wallet.publicKey,
        gameConfig: gameConfigPda,
        mint0: resourceMints[0].publicKey,
        mint1: resourceMints[1].publicKey,
        mint2: resourceMints[2].publicKey,
        mint3: resourceMints[3].publicKey,
        mint4: resourceMints[4].publicKey,
        mint5: resourceMints[5].publicKey,
        ata0: playerAtas[0],
        ata1: playerAtas[1],
        ata2: playerAtas[2],
        ata3: playerAtas[3],
        ata4: playerAtas[4],
        ata5: playerAtas[5],
        cpiAuth,
        resourceMintAuth,
        resourceManagerProgram: rmProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
      .rpc();

    // Sum of all resource token balances should be 3 (one per search call).
    // Use context.banksClient.getAccount() — getTokenAccountBalance is not available on BanksClient.
    // Token account amount is at bytes 64–71 (u64 LE), same layout for Token-2022 ATAs.
    let total = 0;
    for (const kp of resourceMints) {
      const ata = getAssociatedTokenAddressSync(kp.publicKey, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID);
      const account = await context.banksClient.getAccount(ata);
      if (account && account.data.length >= 72) {
        total += Number(Buffer.from(account.data).readBigUInt64LE(64));
      }
    }
    assert.equal(total, 3, "Should have received 3 resource tokens total");
  });

  it("fails cooldown on immediate second search", async () => {
    const [resourceMintAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("resource_mint_auth")],
      rmProgram.programId
    );
    const [cpiAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("cpi_auth")],
      searchProgram.programId
    );
    const playerAtas = resourceMints.map((kp) =>
      getAssociatedTokenAddressSync(kp.publicKey, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID)
    );

    try {
      await searchProgram.methods
        .searchResources()
        .accounts({
          playerWallet: provider.wallet.publicKey,
          player: playerPda,
          owner: provider.wallet.publicKey,
          gameConfig: gameConfigPda,
          mint0: resourceMints[0].publicKey,
          mint1: resourceMints[1].publicKey,
          mint2: resourceMints[2].publicKey,
          mint3: resourceMints[3].publicKey,
          mint4: resourceMints[4].publicKey,
          mint5: resourceMints[5].publicKey,
          ata0: playerAtas[0],
          ata1: playerAtas[1],
          ata2: playerAtas[2],
          ata3: playerAtas[3],
          ata4: playerAtas[4],
          ata5: playerAtas[5],
          cpiAuth,
          resourceMintAuth,
          resourceManagerProgram: rmProgram.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
        .rpc();
      assert.fail("Should have failed cooldown check");
    } catch (e: any) {
      assert.include(e.message, "CooldownNotElapsed");
    }
  });

  it("succeeds after advancing clock 61 seconds", async () => {
    const currentClock = await context.banksClient.getClock();
    await context.setClock(
      new Clock(
        currentClock.slot,
        currentClock.epochStartTimestamp,
        currentClock.epoch,
        currentClock.leaderScheduleEpoch,
        currentClock.unixTimestamp + 61n
      )
    );

    const [resourceMintAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("resource_mint_auth")],
      rmProgram.programId
    );
    const [cpiAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("cpi_auth")],
      searchProgram.programId
    );
    const playerAtas = resourceMints.map((kp) =>
      getAssociatedTokenAddressSync(kp.publicKey, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID)
    );

    await searchProgram.methods
      .searchResources()
      .accounts({
        playerWallet: provider.wallet.publicKey,
        player: playerPda,
        owner: provider.wallet.publicKey,
        gameConfig: gameConfigPda,
        mint0: resourceMints[0].publicKey,
        mint1: resourceMints[1].publicKey,
        mint2: resourceMints[2].publicKey,
        mint3: resourceMints[3].publicKey,
        mint4: resourceMints[4].publicKey,
        mint5: resourceMints[5].publicKey,
        ata0: playerAtas[0],
        ata1: playerAtas[1],
        ata2: playerAtas[2],
        ata3: playerAtas[3],
        ata4: playerAtas[4],
        ata5: playerAtas[5],
        cpiAuth,
        resourceMintAuth,
        resourceManagerProgram: rmProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
      .rpc();

    // Total resource tokens should now be 6 (3+3).
    let total = 0;
    for (const kp of resourceMints) {
      const ata = getAssociatedTokenAddressSync(kp.publicKey, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID);
      const account = await context.banksClient.getAccount(ata);
      if (account && account.data.length >= 72) {
        total += Number(Buffer.from(account.data).readBigUInt64LE(64));
      }
    }
    assert.equal(total, 6, "Should have 6 total resource tokens after two searches");
  });

  it("rejects search by wrong wallet (not player owner)", async () => {
    const wrongWallet = Keypair.generate();
    const [cpiAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("cpi_auth")],
      searchProgram.programId
    );
    const [resourceMintAuth] = PublicKey.findProgramAddressSync(
      [Buffer.from("resource_mint_auth")],
      rmProgram.programId
    );
    const playerAtas = resourceMints.map((kp) =>
      getAssociatedTokenAddressSync(kp.publicKey, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID)
    );

    try {
      await searchProgram.methods
        .searchResources()
        .accounts({
          playerWallet: wrongWallet.publicKey,
          player: playerPda,
          owner: provider.wallet.publicKey,
          gameConfig: gameConfigPda,
          mint0: resourceMints[0].publicKey,
          mint1: resourceMints[1].publicKey,
          mint2: resourceMints[2].publicKey,
          mint3: resourceMints[3].publicKey,
          mint4: resourceMints[4].publicKey,
          mint5: resourceMints[5].publicKey,
          ata0: playerAtas[0],
          ata1: playerAtas[1],
          ata2: playerAtas[2],
          ata3: playerAtas[3],
          ata4: playerAtas[4],
          ata5: playerAtas[5],
          cpiAuth,
          resourceMintAuth,
          resourceManagerProgram: rmProgram.programId,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([wrongWallet])
        .rpc();
      assert.fail("Should have rejected wrong wallet");
    } catch (e: any) {
      assert.match(e.message, /ConstraintSeeds|Unauthorized/);
    }
  });
});
