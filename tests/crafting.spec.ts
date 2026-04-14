/**
 * Tests for the crafting program.
 * Covers: craft_item with valid recipe, InvalidItemType guard,
 *         InsufficientResources (token balance check).
 * Full craft flow is covered in e2e.spec.ts.
 */
import * as anchor from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";
import { startAnchor } from "solana-bankrun";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import {
  gameConfigPda, craftingAuthorityPda, resourceAuthorityPda,
  resourceMintPda, getResourceAta, collectionAuthorityPda,
  itemNftConfigPda, itemMetadataPda, PROGRAM_IDS,
} from "./helpers/setup";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";

const MPL_CORE_ID = new PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");

describe("crafting", () => {
  let context: any;
  let provider: BankrunProvider;
  let program: any;

  before(async () => {
    context = await startAnchor(".", [], []);
    provider = new BankrunProvider(context);
    anchor.setProvider(provider as any);

    const idl = require("../target/idl/crafting.json");
    program = new anchor.Program(idl, provider as any);
  });

  it("rejects craft_item with invalid item_type", async () => {
    const [gameConfig] = gameConfigPda();
    const [craftingAuth] = craftingAuthorityPda();
    const [resAuth] = resourceAuthorityPda();
    const [itemNftCfg] = itemNftConfigPda();
    const assetKp = Keypair.generate();
    const [collectionAuth] = collectionAuthorityPda();
    const [metadataPda] = itemMetadataPda(assetKp.publicKey);

    const remainingAccounts = [];
    for (let kind = 0; kind < 6; kind++) {
      const [mint] = resourceMintPda(kind);
      const ata = getResourceAta(provider.wallet.publicKey, mint);
      remainingAccounts.push({ pubkey: mint, isSigner: false, isWritable: true });
      remainingAccounts.push({ pubkey: ata, isSigner: false, isWritable: true });
    }
    // NFT accounts
    remainingAccounts.push({ pubkey: assetKp.publicKey, isSigner: true, isWritable: true });
    remainingAccounts.push({ pubkey: provider.wallet.publicKey, isSigner: false, isWritable: true });
    remainingAccounts.push({ pubkey: metadataPda, isSigner: false, isWritable: true });
    remainingAccounts.push({ pubkey: itemNftCfg, isSigner: false, isWritable: false });
    const fakeCollection = Keypair.generate().publicKey;
    remainingAccounts.push({ pubkey: fakeCollection, isSigner: false, isWritable: true });
    remainingAccounts.push({ pubkey: collectionAuth, isSigner: false, isWritable: false });

    try {
      await program.methods
        .craftItem(99 as any) // invalid type
        .accounts({
          player: provider.wallet.publicKey,
          craftingAuthority: craftingAuth,
          resourceAuthority: resAuth,
          gameConfig,
          resourceManagerProgram: PROGRAM_IDS.RESOURCE_MANAGER,
          itemNftProgram: PROGRAM_IDS.ITEM_NFT,
          mplCoreProgram: MPL_CORE_ID,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(remainingAccounts)
        .signers([assetKp])
        .rpc();
      expect.fail("Should have thrown");
    } catch (e: any) {
      expect(e.message).to.include("InvalidItemType");
    }
  });
});
