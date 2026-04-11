import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import { MagicToken } from "../../../target/types/magic_token";
import { loadAccounts } from "../../../utils/account_utils";

describe("magic_token", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.MagicToken as Program<MagicToken>;

  it("rejects mint_magic_token without marketplace cpi_auth", async () => {
    const saved = loadAccounts();
    const magicTokenMint = new PublicKey(saved["magicTokenMint"]);
    const [magicMintAuth] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("magic_mint_auth")],
      program.programId
    );

    const fakeAuth = provider.wallet.publicKey;
    try {
      await program.methods
        .mintMagicToken(new anchor.BN(10))
        .accounts({
          cpiAuth: fakeAuth,
          mint: magicTokenMint,
          recipientAta: provider.wallet.publicKey, // placeholder, irrelevant since it will fail before
          recipient: provider.wallet.publicKey,
          magicMintAuth,
          payer: provider.wallet.publicKey,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should have rejected unauthorized mint");
    } catch (e: any) {
      assert.include(e.message, "Unauthorized");
    }
  });
});
