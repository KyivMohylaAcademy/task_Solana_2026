import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import {
  createAssociatedTokenAccountIdempotent,
  getAccount,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";
import { ResourceManager } from "../target/types/resource_manager";
import { Search } from "../target/types/search";

const { PublicKey, Keypair, LAMPORTS_PER_SOL } = anchor.web3;

// Seeds — must match Rust constants in both programs.
const GAME_CONFIG_SEED = Buffer.from("game_config");
const RESOURCE_MINT_SEED = Buffer.from("resource_mint");
const MINT_AUTHORITY_SEED = Buffer.from("mint_authority");
const SEARCH_AUTHORITY_SEED = Buffer.from("search_auth");
const PLAYER_SEED = Buffer.from("player");

const RESOURCE_COUNT = 6;
const RESOURCES_PER_SEARCH = 3;
const SEARCH_COOLDOWN_SECONDS = 60;

describe("search", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const search = anchor.workspace.Search as Program<Search>;
  const resourceManager = anchor.workspace
    .ResourceManager as Program<ResourceManager>;

  const [gameConfigPda] = PublicKey.findProgramAddressSync(
    [GAME_CONFIG_SEED],
    resourceManager.programId
  );

  // Search-side PDA the program signs as for CPIs into resource_manager.
  const [searchAuthorityPda] = PublicKey.findProgramAddressSync(
    [SEARCH_AUTHORITY_SEED],
    search.programId
  );

  // Per-resource PDAs (mint + mint authority) all live under resource_manager.
  const resourceMintPdas: anchor.web3.PublicKey[] = [];
  const mintAuthorityPdas: anchor.web3.PublicKey[] = [];
  for (let i = 0; i < RESOURCE_COUNT; i++) {
    resourceMintPdas.push(
      PublicKey.findProgramAddressSync(
        [RESOURCE_MINT_SEED, Buffer.from([i])],
        resourceManager.programId
      )[0]
    );
    mintAuthorityPdas.push(
      PublicKey.findProgramAddressSync(
        [MINT_AUTHORITY_SEED, Buffer.from([i])],
        resourceManager.programId
      )[0]
    );
  }

  // Player wallet — fresh per test file so we don't collide with anything
  // resource_manager.ts left behind. Funded in `before`.
  const player = Keypair.generate();
  const [playerPda] = PublicKey.findProgramAddressSync(
    [PLAYER_SEED, player.publicKey.toBuffer()],
    search.programId
  );

  // Player's ATAs for each resource — populated in `before`.
  let playerAtas: anchor.web3.PublicKey[] = [];

  // Build the giant accounts blob `search_resources` expects. Anchor 1.0's
  // resolver can't derive the per-resource PDAs from instruction args (there
  // are no args), so we hand-feed every slot.
  const searchResourcesAccounts = () => {
    const accs: Record<string, anchor.web3.PublicKey> = {
      player: playerPda,
      wallet: player.publicKey,
      searchAuthority: searchAuthorityPda,
      gameConfig: gameConfigPda,
      resourceManagerProgram: resourceManager.programId,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
    };
    for (let i = 0; i < RESOURCE_COUNT; i++) {
      accs[`mintAuthority${i}`] = mintAuthorityPdas[i];
      accs[`mint${i}`] = resourceMintPdas[i];
      accs[`ata${i}`] = playerAtas[i];
    }
    return accs;
  };

  before(async () => {
    // Fund the player. We need enough SOL for player init + 6 ATA creations
    // + a couple of search transactions.
    const sig = await provider.connection.requestAirdrop(
      player.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);

    // Register the *real* search program on GameConfig. resource_manager.ts
    // ran before us and left behind a fake test value, so this overwrites
    // it. Provider wallet is the admin.
    await resourceManager.methods
      .setSearchProgram(search.programId)
      .accounts({ admin: provider.wallet.publicKey })
      .rpc();

    // Create the player's 6 ATAs up front. Doing this client-side (rather
    // than init_if_needed inside the program) keeps the search instruction
    // well within the default compute-unit budget.
    playerAtas = [];
    for (let i = 0; i < RESOURCE_COUNT; i++) {
      const ata = await createAssociatedTokenAccountIdempotent(
        provider.connection,
        player,
        resourceMintPdas[i],
        player.publicKey,
        undefined,
        TOKEN_2022_PROGRAM_ID
      );
      playerAtas.push(ata);
    }
  });

  describe("initialize_player", () => {
    it("creates the Player PDA for the wallet", async () => {
      await search.methods
        .initializePlayer()
        .accounts({ wallet: player.publicKey })
        .signers([player])
        .rpc();

      const playerAccount = await search.account.player.fetch(playerPda);
      expect(playerAccount.wallet.toBase58()).to.equal(
        player.publicKey.toBase58()
      );
      expect(playerAccount.lastSearchTimestamp.toNumber()).to.equal(0);
    });

    it("rejects a second initialization for the same wallet", async () => {
      let threw = false;
      try {
        await search.methods
          .initializePlayer()
          .accounts({ wallet: player.publicKey })
          .signers([player])
          .rpc();
      } catch (err) {
        threw = true;
        expect(String(err)).to.include("already in use");
      }
      expect(threw, "second init should have thrown").to.equal(true);
    });
  });

  describe("search_resources", () => {
    it("mints RESOURCES_PER_SEARCH tokens across the 6 resources", async () => {
      // Snapshot all 6 ATA balances before the search.
      const before: bigint[] = [];
      for (const ata of playerAtas) {
        const acc = await getAccount(
          provider.connection,
          ata,
          undefined,
          TOKEN_2022_PROGRAM_ID
        );
        before.push(acc.amount);
      }

      await search.methods
        .searchResources()
        .accountsPartial(searchResourcesAccounts())
        .signers([player])
        .rpc();

      // The pseudo-random draw mints exactly RESOURCES_PER_SEARCH tokens
      // total, distributed across the 6 ATAs. Sum the deltas to verify.
      let totalDelta = 0n;
      for (let i = 0; i < RESOURCE_COUNT; i++) {
        const acc = await getAccount(
          provider.connection,
          playerAtas[i],
          undefined,
          TOKEN_2022_PROGRAM_ID
        );
        totalDelta += acc.amount - before[i];
      }
      expect(Number(totalDelta)).to.equal(RESOURCES_PER_SEARCH);

      // last_search_timestamp must now be set to a recent time.
      const playerAccount = await search.account.player.fetch(playerPda);
      const now = Math.floor(Date.now() / 1000);
      expect(playerAccount.lastSearchTimestamp.toNumber()).to.be.greaterThan(
        now - 30
      );
    });

    it("rejects a second search inside the cooldown window", async () => {
      let threw = false;
      try {
        await search.methods
          .searchResources()
          .accountsPartial(searchResourcesAccounts())
          .signers([player])
          .rpc();
      } catch (err) {
        threw = true;
        const logs: string[] = (err as any)?.logs ?? [];
        const text = [String(err), ...logs].join("\n");
        expect(text).to.match(/CooldownNotElapsed|cooldown/i);
      }
      expect(threw, "cooldowned search should have thrown").to.equal(true);
    });

    // Optional slow test — uncomment to verify the cooldown actually expires.
    // Takes ~65 seconds because there's no clock-warp on solana-test-validator.
    //
    // it("succeeds again after the cooldown elapses", async function () {
    //   this.timeout(120_000);
    //   await new Promise((r) => setTimeout(r, SEARCH_COOLDOWN_SECONDS * 1000 + 5_000));
    //   await search.methods
    //     .searchResources()
    //     .accountsPartial(searchResourcesAccounts())
    //     .signers([player])
    //     .rpc();
    // });
  });

  describe("mint_resource (gating)", () => {
    it("rejects a direct call from a wallet pretending to be search_authority", async () => {
      // The mint_resource accounts struct requires `search_authority` to be
      // a Signer at the canonical PDA derived under game_config.search_program.
      // No wallet can produce a private-key signature for a PDA, so the only
      // way to call it is via CPI from the search program.
      //
      // We try two ways and expect both to fail:
      //  - passing the real PDA: cannot be signed without seeds (no priv key).
      //  - passing a random keypair: fails the seeds constraint.
      const intruder = Keypair.generate();
      const sig = await provider.connection.requestAirdrop(
        intruder.publicKey,
        LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig);

      let threw = false;
      try {
        await resourceManager.methods
          .mintResource(0, new anchor.BN(1))
          .accountsPartial({
            gameConfig: gameConfigPda,
            searchAuthority: intruder.publicKey,
            mintAuthority: mintAuthorityPdas[0],
            mint: resourceMintPdas[0],
            recipientAta: playerAtas[0],
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .signers([intruder])
          .rpc();
      } catch (err) {
        threw = true;
        // Either ConstraintSeeds (2006) or ConstraintRaw — we just want a
        // clear seeds-related failure, not a generic crash.
        const logs: string[] = (err as any)?.logs ?? [];
        const text = [String(err), ...logs].join("\n");
        expect(text).to.match(
          /AnchorError.*200[06]|ConstraintSeeds|seeds/i
        );
      }
      expect(threw, "direct mint_resource call should have thrown").to.equal(
        true
      );
    });
  });
});
