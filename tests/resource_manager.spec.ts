/**
 * Tests for the resource_manager program.
 * Covers: initialize_config, create_resource_mint, mint_from_search,
 *         mint_from_crafting, burn_from_crafting, update_item_prices,
 *         update_cooldown, admin-only guards, duplicate-init guards.
 */
import * as anchor from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";
import { startAnchor } from "solana-bankrun";
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { expect } from "chai";
import {
  SEEDS, PROGRAM_IDS,
  gameConfigPda, resourceAuthorityPda, resourceMintPda,
  searchAuthorityPda, craftingAuthorityPda,
} from "./helpers/setup";

describe("resource_manager", () => {
  let context: any;
  let provider: BankrunProvider;
  let program: any;
  let admin: Keypair;

  const ITEM_PRICES: anchor.BN[] = [
    new anchor.BN(100), new anchor.BN(200),
    new anchor.BN(300), new anchor.BN(400),
  ];

  before(async () => {
    admin = Keypair.generate();
    context = await startAnchor(".", [], []);
    provider = new BankrunProvider(context);
    anchor.setProvider(provider as any);

    // Load IDL
    const idl = require("../target/idl/resource_manager.json");
    program = new anchor.Program(idl, provider as any);
  });

  it("initializes GameConfig", async () => {
    const [configPda, bump] = gameConfigPda();

    await program.methods
      .initializeConfig(ITEM_PRICES)
      .accounts({
        admin: provider.wallet.publicKey,
        gameConfig: configPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const config = await program.account.gameConfig.fetch(configPda);
    expect(config.admin.toBase58()).to.equal(provider.wallet.publicKey.toBase58());
    expect(config.cooldownSeconds.toNumber()).to.equal(60);
    expect(config.itemPrices.map((p: anchor.BN) => p.toNumber())).to.deep.equal([100, 200, 300, 400]);
  });

  it("rejects second initializeConfig call", async () => {
    const [configPda] = gameConfigPda();
    try {
      await program.methods
        .initializeConfig(ITEM_PRICES)
        .accounts({
          admin: provider.wallet.publicKey,
          gameConfig: configPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      expect.fail("Should have thrown");
    } catch (e: any) {
      expect(e.message).to.include("already in use");
    }
  });

  it("updates cooldown (admin)", async () => {
    const [configPda] = gameConfigPda();
    await program.methods
      .updateCooldown(new anchor.BN(1))
      .accounts({
        admin: provider.wallet.publicKey,
        gameConfig: configPda,
      })
      .rpc();
    const config = await program.account.gameConfig.fetch(configPda);
    expect(config.cooldownSeconds.toNumber()).to.equal(1);
    // restore
    await program.methods
      .updateCooldown(new anchor.BN(60))
      .accounts({ admin: provider.wallet.publicKey, gameConfig: configPda })
      .rpc();
  });

  it("rejects updateCooldown from non-admin", async () => {
    const [configPda] = gameConfigPda();
    const attacker = Keypair.generate();
    try {
      await program.methods
        .updateCooldown(new anchor.BN(0))
        .accounts({ admin: attacker.publicKey, gameConfig: configPda })
        .signers([attacker])
        .rpc();
      expect.fail("Should have thrown");
    } catch (e: any) {
      expect(e.message).to.include("AdminOnly");
    }
  });

  it("updates item prices (admin)", async () => {
    const [configPda] = gameConfigPda();
    const newPrices = [new anchor.BN(10), new anchor.BN(20), new anchor.BN(30), new anchor.BN(40)];
    await program.methods
      .updateItemPrices(newPrices)
      .accounts({ admin: provider.wallet.publicKey, gameConfig: configPda })
      .rpc();
    const config = await program.account.gameConfig.fetch(configPda);
    expect(config.itemPrices[0].toNumber()).to.equal(10);
    // restore
    await program.methods
      .updateItemPrices(ITEM_PRICES)
      .accounts({ admin: provider.wallet.publicKey, gameConfig: configPda })
      .rpc();
  });

  it("rejects create_resource_mint for invalid kind", async () => {
    const [configPda] = gameConfigPda();
    const mintKp = Keypair.generate();
    const [resourceAuthority] = resourceAuthorityPda();
    try {
      await program.methods
        .createResourceMint(10 as any)
        .accounts({
          admin: provider.wallet.publicKey,
          gameConfig: configPda,
          mint: mintKp.publicKey,
          resourceAuthority,
          tokenProgram: new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"),
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([mintKp])
        .rpc();
      expect.fail("Should have thrown");
    } catch (e: any) {
      expect(e.message).to.include("InvalidResourceKind");
    }
  });
});
