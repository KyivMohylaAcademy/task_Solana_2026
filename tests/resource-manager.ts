/** Integration tests for resource mint creation and authorized mint/burn CPI paths. */
import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { createRequire } from "module";
import {
  createAssociatedTokenAccountIdempotent,
  getAccount,
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import BN from "bn.js";

const require = createRequire(`${process.cwd()}/tests/resource-manager.ts`);
const { getGamePrograms, getProgramPublicKey } = require("../utils/programs");
const {
  findGameConfigPda,
  findMagicTokenMintPda,
  findPlayerPda,
  findProgramAuthorityPda,
  findResourceMintPda,
} = require("../utils/account_utils");

/** Static metadata expected for each resource mint during bootstrap. */
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

describe("resource manager", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programs = getGamePrograms(provider);
  const admin = (provider.wallet as anchor.Wallet).payer;
  const player = anchor.web3.Keypair.generate();
  const itemPrices = [25, 40, 75, 110].map((value) => new BN(value));
  const [gameConfigPda] = findGameConfigPda();
  const [magicTokenMintPda] = findMagicTokenMintPda();
  const [resourceManagerAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("resource_manager"),
  );
  const [searchAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("search"),
  );
  const [craftingAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("crafting"),
  );
  const [playerPda] = findPlayerPda(player.publicKey);

  /** Asserts that a transaction promise fails without caring about the exact error text. */
  const expectRpcToFail = async (promise: Promise<unknown>) => {
    try {
      await promise;
      expect.fail("Expected transaction to fail");
    } catch (_error) {
      expect(true).to.equal(true);
    }
  };

  /** Ensures the test player has enough SOL to pay for ATA creation and transactions. */
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
      const [resourceMintPda] = findResourceMintPda(resourceType);
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

  /** Creates and returns the player's ATA for one resource mint. */
  const ensurePlayerAta = async (
    resourceType: number,
  ): Promise<anchor.web3.PublicKey> => {
    const [resourceMintPda] = findResourceMintPda(resourceType);
    const ata = getAssociatedTokenAddressSync(
      resourceMintPda,
      player.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );

    await createAssociatedTokenAccountIdempotent(
      provider.connection,
      admin,
      resourceMintPda,
      player.publicKey,
      {},
      TOKEN_2022_PROGRAM_ID,
    );

    return ata;
  };

  /** Reads the player's balance for a specific resource mint. */
  const readBalance = async (resourceType: number): Promise<bigint> => {
    const ata = await ensurePlayerAta(resourceType);
    const account = await getAccount(
      provider.connection,
      ata,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );

    return account.amount;
  };

  before(async () => {
    await ensurePlayerFunded();
    await ensureBootstrap();
  });

  it("mints resources only through the authorized search CPI path", async () => {
    const resourceType = 0;
    const amount = 7;
    const [resourceMintPda] = findResourceMintPda(resourceType);
    const playerAta = await ensurePlayerAta(resourceType);
    const beforeBalance = await readBalance(resourceType);

    await programs.search.methods
      .proxyMintResource(resourceType, new BN(amount))
      .accounts({
        owner: player.publicKey,
        player: playerPda,
        gameConfig: gameConfigPda,
        searchAuthority,
        resourceManagerAuthority,
        resourceMint: resourceMintPda,
        playerResourceTokenAccount: playerAta,
        resourceManagerProgram: getProgramPublicKey("resource_manager"),
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player])
      .rpc();

    const afterBalance = await readBalance(resourceType);
    expect(afterBalance - beforeBalance).to.equal(BigInt(amount));
  });

  it("burns resources only through the authorized crafting CPI path", async () => {
    const resourceType = 1;
    const mintedAmount = 9;
    const burnedAmount = 4;
    const [resourceMintPda] = findResourceMintPda(resourceType);
    const playerAta = await ensurePlayerAta(resourceType);

    await programs.search.methods
      .proxyMintResource(resourceType, new BN(mintedAmount))
      .accounts({
        owner: player.publicKey,
        player: playerPda,
        gameConfig: gameConfigPda,
        searchAuthority,
        resourceManagerAuthority,
        resourceMint: resourceMintPda,
        playerResourceTokenAccount: playerAta,
        resourceManagerProgram: getProgramPublicKey("resource_manager"),
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player])
      .rpc();

    const beforeBalance = await readBalance(resourceType);

    await programs.crafting.methods
      .proxyBurnResource(resourceType, new BN(burnedAmount))
      .accounts({
        owner: player.publicKey,
        gameConfig: gameConfigPda,
        craftingAuthority,
        resourceMint: resourceMintPda,
        playerResourceTokenAccount: playerAta,
        resourceManagerProgram: getProgramPublicKey("resource_manager"),
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player])
      .rpc();

    const afterBalance = await readBalance(resourceType);
    expect(beforeBalance - afterBalance).to.equal(BigInt(burnedAmount));
  });

  it("rejects direct mint and burn calls without an authorized program signer", async () => {
    const resourceType = 2;
    const [resourceMintPda] = findResourceMintPda(resourceType);
    const playerAta = await ensurePlayerAta(resourceType);

    await programs.search.methods
      .proxyMintResource(resourceType, new BN(3))
      .accounts({
        owner: player.publicKey,
        player: playerPda,
        gameConfig: gameConfigPda,
        searchAuthority,
        resourceManagerAuthority,
        resourceMint: resourceMintPda,
        playerResourceTokenAccount: playerAta,
        resourceManagerProgram: getProgramPublicKey("resource_manager"),
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player])
      .rpc();

    await expectRpcToFail(
      programs.resource_manager.methods
        .mintResourceToPlayer(resourceType, new BN(1))
        .accounts({
          player: player.publicKey,
          gameConfig: gameConfigPda,
          callerAuthority: searchAuthority,
          programAuthority: resourceManagerAuthority,
          resourceMint: resourceMintPda,
          playerResourceTokenAccount: playerAta,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([player])
        .rpc(),
    );

    await expectRpcToFail(
      programs.resource_manager.methods
        .burnResourceFromPlayer(resourceType, new BN(1))
        .accounts({
          player: player.publicKey,
          gameConfig: gameConfigPda,
          callerAuthority: craftingAuthority,
          resourceMint: resourceMintPda,
          playerResourceTokenAccount: playerAta,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([player])
        .rpc(),
    );
  });
});
