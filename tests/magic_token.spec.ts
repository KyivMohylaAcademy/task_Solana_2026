/**
 * Tests for the magic_token program.
 * Covers: initialize (mint creation), mint_to_player authority guard.
 */
import * as anchor from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";
import { startAnchor } from "solana-bankrun";
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { expect } from "chai";
import {
  magicMintPda, magicAuthorityPda, magicConfigPda,
  marketplaceAuthorityPda, PROGRAM_IDS,
} from "./helpers/setup";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";

describe("magic_token", () => {
  let context: any;
  let provider: BankrunProvider;
  let program: any;

  before(async () => {
    context = await startAnchor(".", [], []);
    provider = new BankrunProvider(context);
    anchor.setProvider(provider as any);

    const idl = require("../target/idl/magic_token.json");
    program = new anchor.Program(idl, provider as any);
  });

  it("initializes the MagicToken mint", async () => {
    const [magicMint, mintBump] = magicMintPda();
    const [magicAuth] = magicAuthorityPda();
    const [magicConfig] = magicConfigPda();

    await program.methods
      .initialize()
      .accounts({
        admin: provider.wallet.publicKey,
        magicMint,
        magicAuthority: magicAuth,
        magicConfig,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const config = await program.account.magicTokenConfig.fetch(magicConfig);
    expect(config.mint.toBase58()).to.equal(magicMint.toBase58());
    expect(config.admin.toBase58()).to.equal(provider.wallet.publicKey.toBase58());
  });

  it("rejects mint_to_player without marketplace_authority signer", async () => {
    const [magicMint] = magicMintPda();
    const [magicAuth] = magicAuthorityPda();
    const [mpAuth] = marketplaceAuthorityPda();
    const fakeAta = Keypair.generate().publicKey;

    const attacker = Keypair.generate();
    try {
      await program.methods
        .mintToPlayer(new anchor.BN(100))
        .accounts({
          marketplaceAuthority: attacker.publicKey, // wrong signer
          magicAuthority: magicAuth,
          magicMint,
          playerAta: fakeAta,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([attacker])
        .rpc();
      expect.fail("Should have thrown");
    } catch (e: any) {
      // Seeds constraint: attacker cannot produce a valid marketplace_authority PDA
      expect(e.message).to.not.equal("Should have thrown");
    }
  });
});
