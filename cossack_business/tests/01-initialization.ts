import * as anchor from "@coral-xyz/anchor";
import { SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import {
  provider,
  rmProgram,
  mtProgram,
  inProgram,
  searchProgram,
  craftProgram,
  marketProgram,
  admin,
  resourceMints,
  resourceNames,
  resourceSymbols,
  itemPrices,
  rarityWeights,
  searchCooldown,
  initializeAll,
} from "./helpers/setup";
import { TOKEN_2022_PROGRAM_ID } from "./helpers/utils";

describe("01 - Initialization", () => {
  let gameConfigPda: anchor.web3.PublicKey;
  let mintAuthorityPda: anchor.web3.PublicKey;
  let magicConfigPda: anchor.web3.PublicKey;
  let magicMintAuthPda: anchor.web3.PublicKey;
  let itemNftConfigPda: anchor.web3.PublicKey;
  let nftAuthorityPda: anchor.web3.PublicKey;

  before(async () => {
    await initializeAll();
    const setup = require("./helpers/setup");
    gameConfigPda = setup.gameConfigPda;
    mintAuthorityPda = setup.mintAuthorityPda;
    magicConfigPda = setup.magicConfigPda;
    magicMintAuthPda = setup.magicMintAuthPda;
    itemNftConfigPda = setup.itemNftConfigPda;
    nftAuthorityPda = setup.nftAuthorityPda;
  });

  it("initializes game config", async () => {
    await rmProgram.methods
      .initializeGame(
        itemPrices.map((p) => new anchor.BN(p)),
        Buffer.from(rarityWeights),
        new anchor.BN(searchCooldown),
        searchProgram.programId,
        craftProgram.programId,
        marketProgram.programId
      )
      .accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPda,
        mintAuthority: mintAuthorityPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const config = await rmProgram.account.gameConfig.fetch(gameConfigPda);
    expect(config.admin.toBase58()).to.equal(admin.publicKey.toBase58());
    expect(config.searchCooldown.toNumber()).to.equal(searchCooldown);
    expect(config.resourceCount).to.equal(0);
  });

  it("initializes all 6 resource mints", async () => {
    for (let i = 0; i < 6; i++) {
      await rmProgram.methods
        .initializeResource(i, resourceNames[i], resourceSymbols[i], `https://cossack.game/resource/${i}.json`)
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          mint: resourceMints[i].publicKey,
          mintAuthority: mintAuthorityPda,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([resourceMints[i]])
        .rpc();
    }

    const config = await rmProgram.account.gameConfig.fetch(gameConfigPda);
    expect(config.resourceCount).to.equal(6);
    for (let i = 0; i < 6; i++) {
      expect(config.resourceMints[i].toBase58()).to.equal(
        resourceMints[i].publicKey.toBase58()
      );
    }
  });

  it("initializes MagicToken mint", async () => {
    const setup = require("./helpers/setup");
    await mtProgram.methods
      .initializeMagicToken(
        "MagicToken",
        "MAGIC",
        "https://cossack.game/magic.json",
        marketProgram.programId
      )
      .accounts({
        admin: admin.publicKey,
        config: magicConfigPda,
        mint: setup.magicMintKp.publicKey,
        mintAuthority: magicMintAuthPda,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([setup.magicMintKp])
      .rpc();

    const config = await mtProgram.account.magicTokenConfig.fetch(magicConfigPda);
    expect(config.mint.toBase58()).to.equal(setup.magicMintKp.publicKey.toBase58());
  });

  it("initializes item_nft config", async () => {
    await inProgram.methods
      .initializeItemNft(craftProgram.programId, marketProgram.programId)
      .accounts({
        admin: admin.publicKey,
        config: itemNftConfigPda,
        nftAuthority: nftAuthorityPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const config = await inProgram.account.itemNftConfig.fetch(itemNftConfigPda);
    expect(config.craftingProgram.toBase58()).to.equal(
      craftProgram.programId.toBase58()
    );
  });

  it("fails to reinitialize game config", async () => {
    try {
      await rmProgram.methods
        .initializeGame(
          itemPrices.map((p) => new anchor.BN(p)),
          Buffer.from(rarityWeights),
          new anchor.BN(60),
          searchProgram.programId,
          craftProgram.programId,
          marketProgram.programId
        )
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
          mintAuthority: mintAuthorityPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      expect.fail("Should have thrown");
    } catch (err) {
      expect(err).to.exist;
    }
  });

  it("admin updates rarity weights", async () => {
    const newWeights = [25, 25, 20, 15, 10, 5];
    await rmProgram.methods
      .updateRarityWeights(Buffer.from(newWeights))
      .accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPda,
      })
      .rpc();

    const config = await rmProgram.account.gameConfig.fetch(gameConfigPda);
    expect(Array.from(config.rarityWeights)).to.deep.equal(newWeights);

    await rmProgram.methods
      .updateRarityWeights(Buffer.from(rarityWeights))
      .accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPda,
      })
      .rpc();
  });

  it("rejects rarity weights that don't sum to 100", async () => {
    try {
      await rmProgram.methods
        .updateRarityWeights(Buffer.from([10, 10, 10, 10, 10, 10]))
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
        })
        .rpc();
      expect.fail("Should have thrown");
    } catch (err) {
      expect(err.toString()).to.include("InvalidRarityWeights");
    }
  });

  it("admin updates search cooldown", async () => {
    await rmProgram.methods
      .updateSearchCooldown(new anchor.BN(3))
      .accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPda,
      })
      .rpc();

    const config = await rmProgram.account.gameConfig.fetch(gameConfigPda);
    expect(config.searchCooldown.toNumber()).to.equal(3);

    await rmProgram.methods
      .updateSearchCooldown(new anchor.BN(searchCooldown))
      .accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPda,
      })
      .rpc();
  });

  it("rejects search cooldown <= 0", async () => {
    try {
      await rmProgram.methods
        .updateSearchCooldown(new anchor.BN(0))
        .accounts({
          admin: admin.publicKey,
          gameConfig: gameConfigPda,
        })
        .rpc();
      expect.fail("Should have thrown");
    } catch (err) {
      expect(err.toString()).to.include("InvalidCooldown");
    }
  });
});
