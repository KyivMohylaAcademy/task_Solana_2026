import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import {
  getMetadataPointerState,
  getMint,
  getTokenMetadata,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";
import { ResourceManager } from "../target/types/resource_manager";

// `@anchor-lang/core` re-exports `@solana/web3.js` under `anchor.web3`, so we
// pull PublicKey/Keypair from there rather than adding a direct dependency.
const { PublicKey, Keypair } = anchor.web3;

// Seeds — must match `programs/resource_manager/src/constants.rs`.
const GAME_CONFIG_SEED = Buffer.from("game_config");
const RESOURCE_MINT_SEED = Buffer.from("resource_mint");
const MINT_AUTHORITY_SEED = Buffer.from("mint_authority");

const RESOURCE_COUNT = 6;

// Must match `constants.rs::RESOURCE_NAMES` / `RESOURCE_SYMBOLS`. The on-chain
// TokenMetadata extension stores these, so the tests can verify the CPI
// actually wrote what we expected.
const RESOURCE_NAMES = ["Wood", "Iron", "Gold", "Leather", "Stone", "Diamond"];
const RESOURCE_SYMBOLS = ["WOOD", "IRON", "GOLD", "LTHR", "STNE", "DMND"];

describe("resource_manager", () => {
  // `AnchorProvider.env()` reads `ANCHOR_PROVIDER_URL` + `ANCHOR_WALLET` from
  // the environment. `anchor test` sets both, pointing at the local validator
  // and the wallet defined in `Anchor.toml [provider]`.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // `workspace.ResourceManager` is magic: Anchor reads `target/idl/` and
  // returns a typed `Program<ResourceManager>` for the generated IDL.
  const program = anchor.workspace.ResourceManager as Program<ResourceManager>;

  // Derive `GameConfig` PDA exactly the same way the Rust program does.
  const [gameConfigPda, gameConfigBump] = PublicKey.findProgramAddressSync(
    [GAME_CONFIG_SEED],
    program.programId
  );

  // Derive the per-resource mint PDA and mint-authority PDA for an id.
  const deriveResourceMint = (resourceId: number) =>
    PublicKey.findProgramAddressSync(
      [RESOURCE_MINT_SEED, Buffer.from([resourceId])],
      program.programId
    );
  const deriveMintAuthority = (resourceId: number) =>
    PublicKey.findProgramAddressSync(
      [MINT_AUTHORITY_SEED, Buffer.from([resourceId])],
      program.programId
    );

  describe("initialize_game_config", () => {
    it("creates the GameConfig PDA with the caller as admin", async () => {
      // Anchor 1.0 `.accounts()` only takes what can't be auto-resolved.
      // `gameConfig` is derivable from seeds; `systemProgram` is well-known.
      await program.methods
        .initializeGameConfig()
        .accounts({
          admin: provider.wallet.publicKey,
        })
        .rpc();

      const gameConfig = await program.account.gameConfig.fetch(gameConfigPda);
      expect(gameConfig.admin.toBase58()).to.equal(
        provider.wallet.publicKey.toBase58()
      );
      expect(gameConfig.bump).to.equal(gameConfigBump);

      // Fresh GameConfig: every resource slot is zeroed — no mints yet.
      for (const slot of gameConfig.resourceMints) {
        expect(slot.toBase58()).to.equal(PublicKey.default.toBase58());
      }
    });

    it("rejects a second initialization of the same GameConfig", async () => {
      let threw = false;
      try {
        await program.methods
          .initializeGameConfig()
          .accounts({
            admin: provider.wallet.publicKey,
          })
          .rpc();
      } catch (err) {
        threw = true;
        expect(String(err)).to.include("already in use");
      }
      expect(threw, "second init should have thrown").to.equal(true);
    });
  });

  describe("initialize_resource_mint", () => {
    // Tests run in declaration order and share the same validator state, so
    // this order matters:
    //   1. negative test (no mints exist yet — `has_one` is the first check
    //      that can fail)
    //   2. happy path (creates all 6)
    //   3. re-init negative test (relies on resource 0 existing)

    it("rejects a non-admin caller", async () => {
      // `has_one = admin` on GameConfig ties the signer to the stored admin.
      // A fresh keypair can't pass that check. Must run before any mint is
      // created, otherwise Anchor's `init` constraint fails first and we'd
      // be testing the wrong thing.
      const intruder = Keypair.generate();

      // Fund the intruder so their tx can pay fees — otherwise we'd fail
      // for the wrong reason (insufficient lamports, not authority mismatch).
      const airdropSig = await provider.connection.requestAirdrop(
        intruder.publicKey,
        1_000_000_000
      );
      await provider.connection.confirmTransaction(airdropSig);

      const [expectedMint] = deriveResourceMint(0);
      const [expectedAuthority] = deriveMintAuthority(0);

      let threw = false;
      try {
        await program.methods
          .initializeResourceMint(0)
          .accountsPartial({
            admin: intruder.publicKey,
            mint: expectedMint,
            mintAuthority: expectedAuthority,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .signers([intruder])
          .rpc();
      } catch (err) {
        threw = true;
        const logs: string[] = (err as any)?.logs ?? [];
        const text = [String(err), ...logs].join("\n");
        // Anchor's has_one constraint violation: error code 2001.
        expect(text).to.match(/AnchorError.*2001|ConstraintHasOne|has[_ ]one/i);
      }
      expect(threw, "non-admin call should have thrown").to.equal(true);
    });

    it("creates all 6 resource mints with PDA authorities", async () => {
      for (let resourceId = 0; resourceId < RESOURCE_COUNT; resourceId++) {
        const [expectedMint] = deriveResourceMint(resourceId);
        const [expectedAuthority] = deriveMintAuthority(resourceId);

        // Anchor 1.0's auto-resolver can't derive PDAs whose seeds depend on
        // instruction arguments, so we pass `mint` and `mintAuthority`
        // explicitly. `.accountsPartial` allows passing the extras while
        // still letting Anchor fill in `gameConfig` and `systemProgram`.
        await program.methods
          .initializeResourceMint(resourceId)
          .accountsPartial({
            admin: provider.wallet.publicKey,
            mint: expectedMint,
            mintAuthority: expectedAuthority,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .rpc();

        // Fetch on-chain mint data via spl-token — verifies that the
        // Token-2022 CPI actually ran and wrote sane state.
        const mintInfo = await getMint(
          provider.connection,
          expectedMint,
          undefined,
          TOKEN_2022_PROGRAM_ID
        );
        expect(mintInfo.decimals).to.equal(0);
        expect(mintInfo.supply.toString()).to.equal("0");
        expect(mintInfo.mintAuthority!.toBase58()).to.equal(
          expectedAuthority.toBase58()
        );

        // MetadataPointer extension: pointer must target the mint itself
        // (self-referential), and update authority is our PDA.
        const pointerState = getMetadataPointerState(mintInfo);
        expect(pointerState, "mint missing MetadataPointer extension").to.not
          .equal(null);
        expect(pointerState!.metadataAddress!.toBase58()).to.equal(
          expectedMint.toBase58()
        );
        expect(pointerState!.authority!.toBase58()).to.equal(
          expectedAuthority.toBase58()
        );

        // TokenMetadata extension: name/symbol written by our second CPI.
        // `getTokenMetadata` fetches straight from the mint because the
        // pointer above says "metadata lives on the mint".
        const metadata = await getTokenMetadata(
          provider.connection,
          expectedMint,
          undefined,
          TOKEN_2022_PROGRAM_ID
        );
        expect(metadata, "mint missing TokenMetadata extension").to.not.equal(
          null
        );
        expect(metadata!.name).to.equal(RESOURCE_NAMES[resourceId]);
        expect(metadata!.symbol).to.equal(RESOURCE_SYMBOLS[resourceId]);
        expect(metadata!.uri).to.equal("");
        expect(metadata!.mint.toBase58()).to.equal(expectedMint.toBase58());
      }

      // After all 6 calls, GameConfig should list every mint address.
      const gameConfig = await program.account.gameConfig.fetch(gameConfigPda);
      for (let resourceId = 0; resourceId < RESOURCE_COUNT; resourceId++) {
        const [expectedMint] = deriveResourceMint(resourceId);
        expect(gameConfig.resourceMints[resourceId].toBase58()).to.equal(
          expectedMint.toBase58()
        );
      }
    });

    it("rejects re-initialization of an existing resource mint", async () => {
      // Resource 0 was initialised in the previous test. Trying again should
      // fail at Anchor's `init` constraint ("already in use").
      const [expectedMint] = deriveResourceMint(0);
      const [expectedAuthority] = deriveMintAuthority(0);

      let threw = false;
      try {
        await program.methods
          .initializeResourceMint(0)
          .accountsPartial({
            admin: provider.wallet.publicKey,
            mint: expectedMint,
            mintAuthority: expectedAuthority,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
      } catch (err) {
        threw = true;
        expect(String(err)).to.include("already in use");
      }
      expect(threw, "re-init should have thrown").to.equal(true);
    });
  });

  describe("set_search_program", () => {
    it("rejects a non-admin caller", async () => {
      const intruder = Keypair.generate();
      const airdropSig = await provider.connection.requestAirdrop(
        intruder.publicKey,
        1_000_000_000
      );
      await provider.connection.confirmTransaction(airdropSig);

      let threw = false;
      try {
        await program.methods
          .setSearchProgram(Keypair.generate().publicKey)
          .accounts({ admin: intruder.publicKey })
          .signers([intruder])
          .rpc();
      } catch (err) {
        threw = true;
        const logs: string[] = (err as any)?.logs ?? [];
        const text = [String(err), ...logs].join("\n");
        expect(text).to.match(/AnchorError.*2001|ConstraintHasOne|has[_ ]one/i);
      }
      expect(threw, "non-admin call should have thrown").to.equal(true);
    });

    it("admin can register an arbitrary search program id", async () => {
      // Use an arbitrary key for this test. The search-program test file
      // overwrites this with the actual `search` program id in its before
      // hook, so we don't break the search flow.
      const fakeSearchProgram = Keypair.generate().publicKey;

      await program.methods
        .setSearchProgram(fakeSearchProgram)
        .accounts({ admin: provider.wallet.publicKey })
        .rpc();

      const gameConfig = await program.account.gameConfig.fetch(gameConfigPda);
      expect(gameConfig.searchProgram.toBase58()).to.equal(
        fakeSearchProgram.toBase58()
      );
    });
  });
});
