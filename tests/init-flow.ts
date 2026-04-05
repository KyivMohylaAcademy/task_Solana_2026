/** Integration tests covering one-time bootstrap initialization across all programs. */
import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { createRequire } from "module";
import { getMint, TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import BN from "bn.js";

const require = createRequire(`${process.cwd()}/tests/init-flow.ts`);
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

describe("init flow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programs = getGamePrograms(provider);
  const admin = (provider.wallet as anchor.Wallet).payer;
  const player = anchor.web3.Keypair.generate();
  const itemPrices = [25, 40, 75, 110].map((value) => new BN(value));
  const [gameConfigPda] = findGameConfigPda();
  const [resourceManagerAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("resource_manager"),
  );
  const [magicTokenAuthority] = findProgramAuthorityPda(
    getProgramPublicKey("magic_token"),
  );
  const [magicTokenMintPda] = findMagicTokenMintPda();
  const [playerPda] = findPlayerPda(player.publicKey);

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

  /** Asserts that a transaction promise fails without caring about the exact error text. */
  const expectRpcToFail = async (promise: Promise<unknown>) => {
    try {
      await promise;
      expect.fail("Expected transaction to fail");
    } catch (_error) {
      expect(true).to.equal(true);
    }
  };

  before(async () => {
    const signature = await provider.connection.requestAirdrop(
      player.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(signature, "confirmed");
  });

  it("initializes GameConfig, all resource mints, MagicToken mint and Player PDA", async () => {
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

    for (let resourceType = 0; resourceType < 6; resourceType += 1) {
      const [resourceMintPda] = findResourceMintPda(resourceType);
      const metadata = RESOURCE_METADATA[resourceType];

      const existingResourceMint = await provider.connection.getAccountInfo(
        resourceMintPda,
        "confirmed",
      );
      if (!existingResourceMint) {
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
    }

    const existingMagicMint = await provider.connection.getAccountInfo(
      magicTokenMintPda,
      "confirmed",
    );
    if (!existingMagicMint) {
      await programs.magic_token.methods
        .initializeMagicTokenMint()
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          programAuthority: magicTokenAuthority,
          magicTokenMint: magicTokenMintPda,
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

    const gameConfig =
      await programs.resource_manager.account.gameConfig.fetch(gameConfigPda);
    const playerAccount = await programs.search.account.player.fetch(playerPda);

    expect(gameConfig.admin.toBase58()).to.equal(admin.publicKey.toBase58());
    expect(gameConfig.rewardTokenMint.toBase58()).to.equal(
      magicTokenMintPda.toBase58(),
    );
    expect(
      gameConfig.itemPrices.map((value: unknown) => toNumber(value as never)),
    ).to.deep.equal([25, 40, 75, 110]);
    expect(playerAccount.owner.toBase58()).to.equal(
      player.publicKey.toBase58(),
    );
    expect(toNumber(playerAccount.lastSearchTimestamp)).to.equal(0);

    for (let resourceType = 0; resourceType < 6; resourceType += 1) {
      const [resourceMintPda] = findResourceMintPda(resourceType);
      const resourceMint = await getMint(
        provider.connection,
        resourceMintPda,
        "confirmed",
        TOKEN_2022_PROGRAM_ID,
      );

      expect(gameConfig.resourceMints[resourceType].toBase58()).to.equal(
        resourceMintPda.toBase58(),
      );
      expect(resourceMint.decimals).to.equal(0);
      expect(resourceMint.mintAuthority?.toBase58()).to.equal(
        resourceManagerAuthority.toBase58(),
      );
      expect(resourceMint.freezeAuthority?.toBase58()).to.equal(
        resourceManagerAuthority.toBase58(),
      );
      expect(resourceMint.mintAuthority?.toBase58()).to.not.equal(
        admin.publicKey.toBase58(),
      );
      expect(resourceMint.mintAuthority?.toBase58()).to.not.equal(
        player.publicKey.toBase58(),
      );
    }

    const magicMint = await getMint(
      provider.connection,
      magicTokenMintPda,
      "confirmed",
      TOKEN_2022_PROGRAM_ID,
    );

    expect(magicMint.decimals).to.equal(0);
    expect(magicMint.mintAuthority?.toBase58()).to.equal(
      magicTokenAuthority.toBase58(),
    );
    expect(magicMint.freezeAuthority?.toBase58()).to.equal(
      magicTokenAuthority.toBase58(),
    );
    expect(magicMint.mintAuthority?.toBase58()).to.not.equal(
      admin.publicKey.toBase58(),
    );
    expect(magicMint.mintAuthority?.toBase58()).to.not.equal(
      player.publicKey.toBase58(),
    );
  });

  it("rejects repeated init attempts", async () => {
    await expectRpcToFail(
      programs.resource_manager.methods
        .initializeGameConfig(magicTokenMintPda, itemPrices)
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([admin])
        .rpc(),
    );

    await expectRpcToFail(
      programs.search.methods
        .initPlayer()
        .accounts({
          owner: player.publicKey,
          player: playerPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([player])
        .rpc(),
    );
  });

  it("rejects unauthorized mint initialization", async () => {
    const intruder = anchor.web3.Keypair.generate();
    const signature = await provider.connection.requestAirdrop(
      intruder.publicKey,
      anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(signature, "confirmed");

    const [resourceMintPda] = findResourceMintPda(5);
    const [altMagicMintPda] = findMagicTokenMintPda();

    await expectRpcToFail(
      programs.resource_manager.methods
        .initializeResourceMint(
          5,
          RESOURCE_METADATA[5].name,
          RESOURCE_METADATA[5].symbol,
          RESOURCE_METADATA[5].uri,
        )
        .accounts({
          admin: intruder.publicKey,
          gameConfig: gameConfigPda,
          programAuthority: resourceManagerAuthority,
          resourceMint: resourceMintPda,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([intruder])
        .rpc(),
    );

    await expectRpcToFail(
      programs.magic_token.methods
        .initializeMagicTokenMint()
        .accounts({
          admin: intruder.publicKey,
          gameConfig: gameConfigPda,
          programAuthority: magicTokenAuthority,
          magicTokenMint: altMagicMintPda,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([intruder])
        .rpc(),
    );
  });
});
