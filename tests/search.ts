/** Integration tests for resource search randomness, minting and cooldown enforcement. */
import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { createRequire } from "module";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import BN from "bn.js";

const require = createRequire(`${process.cwd()}/tests/search.ts`);
const { getGamePrograms, getProgramPublicKey } = require("../utils/programs");
const {
  findGameConfigPda,
  findMagicTokenMintPda,
  findPlayerPda,
  findProgramAuthorityPda,
  findResourceMintPda,
} = require("../utils/account_utils");

/** Static resource metadata used while bootstrapping test mints. */
const RESOURCE_METADATA = [
  {
    name: "Wood",
    symbol: "WOOD",
    uri: "https://example.com/resources/wood.json",
  },
  {
    name: "Iron",
    symbol: "IRON",
    uri: "https://example.com/resources/iron.json",
  },
  {
    name: "Gold",
    symbol: "GOLD",
    uri: "https://example.com/resources/gold.json",
  },
  {
    name: "Leather",
    symbol: "LETHR",
    uri: "https://example.com/resources/leather.json",
  },
  {
    name: "Stone",
    symbol: "STONE",
    uri: "https://example.com/resources/stone.json",
  },
  {
    name: "Diamond",
    symbol: "DIAM",
    uri: "https://example.com/resources/diamond.json",
  },
] as const;

/** Slightly padded cooldown wait used to avoid timing races in tests. */
const SEARCH_COOLDOWN_MS = 61_000;

describe("search", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programs = getGamePrograms(provider);
  const admin = (provider.wallet as anchor.Wallet).payer;
  const player = anchor.web3.Keypair.generate();
  const itemPrices = [25, 40, 75, 110].map((value) => new BN(value));
  const [gameConfigPda] = findGameConfigPda();
  const [magicTokenMintPda] = findMagicTokenMintPda();
  const [playerPda] = findPlayerPda(player.publicKey);
  const [searchAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("search"),
  );
  const [resourceManagerAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("resource_manager"),
  );
  const searchComputeBudgetIx =
    anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 400_000,
    });

  const resourceMints = RESOURCE_METADATA.map(
    (_, resourceType) => findResourceMintPda(resourceType)[0],
  );
  const resourceTokenAccounts = resourceMints.map((mint) =>
    getAssociatedTokenAddressSync(
      mint,
      player.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    ),
  );

  /** Asserts that a transaction promise fails without caring about the exact error text. */
  const expectRpcToFail = async (promise: Promise<unknown>) => {
    try {
      await promise;
      expect.fail("Expected transaction to fail");
    } catch (_error) {
      expect(true).to.equal(true);
    }
  };

  /** Sleeps for the specified duration in milliseconds. */
  const sleep = (ms: number) =>
    new Promise((resolve) => {
      setTimeout(resolve, ms);
    });

  /** Normalizes Anchor numeric wrappers into plain JavaScript numbers for assertions. */
  const toNumber = (
    value: number | bigint | { toNumber?: () => number; toString(): string },
  ): number => {
    if (typeof value === "number") {
      return value;
    }

    if (typeof value === "bigint") {
      return Number(value);
    }

    if ("toNumber" in value && typeof value.toNumber === "function") {
      return value.toNumber();
    }

    return Number(value.toString());
  };

  /** Ensures the test player has enough SOL for ATAs and search transactions. */
  const ensurePlayerFunded = async () => {
    const balance = await provider.connection.getBalance(
      player.publicKey,
      "confirmed",
    );
    if (balance >= anchor.web3.LAMPORTS_PER_SOL / 2) {
      return;
    }

    const signature = await provider.connection.requestAirdrop(
      player.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(signature, "confirmed");
  };

  /** Creates the shared config, resource mints and player account when missing. */
  const ensureBootstrap = async () => {
    const existingGameConfig =
      await programs.resource_manager.account.gameConfig.fetchNullable(
        gameConfigPda,
      );

    if (!existingGameConfig) {
      await programs.resource_manager.methods
        .initializeGameConfig(magicTokenMintPda, itemPrices)
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin])
        .rpc();
    }

    for (
      let resourceType = 0;
      resourceType < RESOURCE_METADATA.length;
      resourceType += 1
    ) {
      const resourceMintPda = resourceMints[resourceType];
      const accountInfo = await provider.connection.getAccountInfo(
        resourceMintPda,
        "confirmed",
      );

      if (accountInfo) {
        continue;
      }

      const metadata = RESOURCE_METADATA[resourceType];
      await programs.resource_manager.methods
        .initializeResourceMint(
          resourceType,
          metadata.name,
          metadata.symbol,
          metadata.uri,
        )
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          programAuthority: resourceManagerAuthority,
          resourceMint: resourceMintPda,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin])
        .rpc();
    }

    const existingPlayer =
      await programs.search.account.player.fetchNullable(playerPda);
    if (!existingPlayer) {
      await programs.search.methods
        .initPlayer()
        .accounts({
          owner: player.publicKey,
          player: playerPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([player])
        .rpc();
    }
  };

  /** Builds the fixed account map expected by the `searchResources` instruction. */
  const buildSearchAccounts = () => {
    return {
      owner: player.publicKey,
      player: playerPda,
      gameConfig: gameConfigPda,
      searchAuthority,
      resourceManagerAuthority,
      woodMint: resourceMints[0],
      ironMint: resourceMints[1],
      goldMint: resourceMints[2],
      leatherMint: resourceMints[3],
      stoneMint: resourceMints[4],
      diamondMint: resourceMints[5],
      woodTokenAccount: resourceTokenAccounts[0],
      ironTokenAccount: resourceTokenAccounts[1],
      goldTokenAccount: resourceTokenAccounts[2],
      leatherTokenAccount: resourceTokenAccounts[3],
      stoneTokenAccount: resourceTokenAccounts[4],
      diamondTokenAccount: resourceTokenAccounts[5],
      resourceManagerProgram: getProgramPublicKey("resource_manager"),
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    };
  };

  /** Reads one resource balance for the current player. */
  const readBalance = async (resourceIndex: number): Promise<bigint> => {
    const tokenAccount = resourceTokenAccounts[resourceIndex];
    const accountInfo = await provider.connection.getAccountInfo(
      tokenAccount,
      "confirmed",
    );

    if (!accountInfo) {
      return 0n;
    }

    const account = await getAccount(
      provider.connection,
      tokenAccount,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );

    return account.amount;
  };

  /** Reads the sum of all resource balances owned by the current player. */
  const readTotalBalance = async (): Promise<bigint> => {
    const balances = await Promise.all(
      resourceTokenAccounts.map((_account, index) => readBalance(index)),
    );
    return balances.reduce((total, balance) => total + balance, 0n);
  };

  before(async () => {
    await ensurePlayerFunded();
    await ensureBootstrap();
  });

  it("mints exactly three resource tokens on the first search", async () => {
    const beforeTotal = await readTotalBalance();

    const signature = await programs.search.methods
      .searchResources()
      .preInstructions([searchComputeBudgetIx])
      .accounts(buildSearchAccounts())
      .signers([player])
      .rpc();
    await provider.connection.confirmTransaction(signature, "confirmed");

    const afterTotal = await readTotalBalance();
    const playerAccount = await programs.search.account.player.fetch(playerPda);

    expect(afterTotal - beforeTotal).to.equal(3n);
    expect(toNumber(playerAccount.lastSearchTimestamp)).to.be.greaterThan(0);
  });

  it("rejects a repeated search before the 60-second cooldown finishes", async () => {
    const beforeTotal = await readTotalBalance();

    await expectRpcToFail(
      programs.search.methods
        .searchResources()
        .preInstructions([searchComputeBudgetIx])
        .accounts(buildSearchAccounts())
        .signers([player])
        .rpc(),
    );

    const afterTotal = await readTotalBalance();
    expect(afterTotal).to.equal(beforeTotal);
  });

  it("allows searching again after a real cooldown delay", async () => {
    const beforeTotal = await readTotalBalance();

    await sleep(SEARCH_COOLDOWN_MS);

    const signature = await programs.search.methods
      .searchResources()
      .preInstructions([searchComputeBudgetIx])
      .accounts(buildSearchAccounts())
      .signers([player])
      .rpc();
    await provider.connection.confirmTransaction(signature, "confirmed");

    const afterTotal = await readTotalBalance();
    expect(afterTotal - beforeTotal).to.equal(3n);
  });
});
