import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import { ResourceManager } from "../../../target/types/resource_manager";
import { saveAccounts } from "../../../utils/account_utils";

const RESOURCE_NAMES = ["Wood", "Iron", "Gold", "Leather", "Stone", "Diamond"];
const RESOURCE_SYMBOLS = ["WOOD", "IRON", "GOLD", "LTHR", "STON", "DIAM"];
const RESOURCE_URIS = [
  "https://REPLACE_ME/wood.json",
  "https://REPLACE_ME/iron.json",
  "https://REPLACE_ME/gold.json",
  "https://REPLACE_ME/leather.json",
  "https://REPLACE_ME/stone.json",
  "https://REPLACE_ME/diamond.json",
];

describe("resource_manager: init_mints", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.ResourceManager as Program<ResourceManager>;

  const mintKeypairs: Keypair[] = Array.from({ length: 6 }, () => Keypair.generate());

  const [resourceMintAuth] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("resource_mint_auth")],
    program.programId
  );

  it("initializes all 6 resource mints", async () => {
    for (let i = 0; i < 6; i++) {
      await program.methods
        .initResourceMint(i, RESOURCE_NAMES[i], RESOURCE_SYMBOLS[i], RESOURCE_URIS[i])
        .accounts({
          payer: provider.wallet.publicKey,
          mint: mintKeypairs[i].publicKey,
          resourceMintAuth,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([mintKeypairs[i]])
        .rpc();
    }

    // Verify all mints exist.
    for (const kp of mintKeypairs) {
      const info = await provider.connection.getAccountInfo(kp.publicKey);
      assert.isNotNull(info, "Mint account should exist");
    }

    // Save mint addresses for downstream tests.
    const accounts: Record<string, string> = {};
    for (let i = 0; i < 6; i++) {
      accounts[`resourceMint${i}`] = mintKeypairs[i].publicKey.toBase58();
    }
    saveAccounts(accounts);
    console.log("Resource mints saved to accounts.json");
  });
});
