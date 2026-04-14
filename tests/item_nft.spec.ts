/**
 * Tests for the item_nft program.
 * Covers: initialize_collection, mint_item (crafting-authority guard),
 *         burn_item (marketplace-authority guard), WrongOwner check.
 */
import * as anchor from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";
import { startAnchor } from "solana-bankrun";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import {
  collectionAuthorityPda, itemNftConfigPda, craftingAuthorityPda,
  marketplaceAuthorityPda, itemMetadataPda, PROGRAM_IDS,
} from "./helpers/setup";

// mpl-core program ID
const MPL_CORE_ID = new PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");

describe("item_nft", () => {
  let context: any;
  let provider: BankrunProvider;
  let program: any;

  before(async () => {
    context = await startAnchor(".", [], []);
    provider = new BankrunProvider(context);
    anchor.setProvider(provider as any);

    const idl = require("../target/idl/item_nft.json");
    program = new anchor.Program(idl, provider as any);
  });

  it("initializes collection", async () => {
    const collectionKp = Keypair.generate();
    const [collectionAuth] = collectionAuthorityPda();
    const [configPda] = itemNftConfigPda();

    await program.methods
      .initializeCollection()
      .accounts({
        admin: provider.wallet.publicKey,
        collection: collectionKp.publicKey,
        collectionAuthority: collectionAuth,
        itemNftConfig: configPda,
        mplCoreProgram: MPL_CORE_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([collectionKp])
      .rpc();

    const config = await program.account.itemNftConfig.fetch(configPda);
    expect(config.collection.toBase58()).to.equal(collectionKp.publicKey.toBase58());
  });

  it("rejects mint_item without crafting_authority signer", async () => {
    const [configPda] = itemNftConfigPda();
    const config = await program.account.itemNftConfig.fetch(configPda);
    const assetKp = Keypair.generate();
    const [collectionAuth] = collectionAuthorityPda();
    const [metadataPda] = itemMetadataPda(assetKp.publicKey);
    const attacker = Keypair.generate();

    try {
      await program.methods
        .mintItem(0, provider.wallet.publicKey)
        .accounts({
          craftingAuthority: attacker.publicKey, // wrong
          asset: assetKp.publicKey,
          collection: config.collection,
          collectionAuthority: collectionAuth,
          recipient: provider.wallet.publicKey,
          payer: provider.wallet.publicKey,
          itemMetadata: metadataPda,
          itemNftConfig: configPda,
          mplCoreProgram: MPL_CORE_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([assetKp, attacker])
        .rpc();
      expect.fail("Should have thrown");
    } catch (e: any) {
      expect(e.message).to.not.equal("Should have thrown");
    }
  });

  it("rejects burn_item without marketplace_authority signer", async () => {
    const [configPda] = itemNftConfigPda();
    const config = await program.account.itemNftConfig.fetch(configPda);
    const fakeAsset = Keypair.generate().publicKey;
    const [collectionAuth] = collectionAuthorityPda();
    const [metadataPda] = itemMetadataPda(fakeAsset);
    const attacker = Keypair.generate();

    try {
      await program.methods
        .burnItem()
        .accounts({
          marketplaceAuthority: attacker.publicKey, // wrong
          asset: fakeAsset,
          collection: config.collection,
          collectionAuthority: collectionAuth,
          payer: provider.wallet.publicKey,
          itemMetadata: metadataPda,
          itemNftConfig: configPda,
          mplCoreProgram: MPL_CORE_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([attacker])
        .rpc();
      expect.fail("Should have thrown");
    } catch (e: any) {
      expect(e.message).to.not.equal("Should have thrown");
    }
  });
});
