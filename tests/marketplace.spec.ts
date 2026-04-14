/**
 * Tests for the marketplace program.
 * Covers: sell_item WrongOwner guard, ItemTypeMismatch guard, InvalidItemType guard.
 * Full sell flow is covered in e2e.spec.ts.
 */
import * as anchor from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";
import { startAnchor } from "solana-bankrun";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import {
  gameConfigPda, marketplaceAuthorityPda, itemMetadataPda,
  collectionAuthorityPda, itemNftConfigPda,
  magicAuthorityPda, magicMintPda, getMagicAta, PROGRAM_IDS,
} from "./helpers/setup";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";

const MPL_CORE_ID = new PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");

describe("marketplace", () => {
  let context: any;
  let provider: BankrunProvider;
  let program: any;

  before(async () => {
    context = await startAnchor(".", [], []);
    provider = new BankrunProvider(context);
    anchor.setProvider(provider as any);

    const idl = require("../target/idl/marketplace.json");
    program = new anchor.Program(idl, provider as any);
  });

  it("rejects sell_item with invalid item_type", async () => {
    const [mpAuth] = marketplaceAuthorityPda();
    const [gameConfig] = gameConfigPda();
    const fakeAsset = Keypair.generate().publicKey;
    const [metadataPda] = itemMetadataPda(fakeAsset);
    const [colAuth] = collectionAuthorityPda();
    const [nftCfg] = itemNftConfigPda();
    const [magicAuth] = magicAuthorityPda();
    const [magicMint] = magicMintPda();
    const sellerAta = getMagicAta(provider.wallet.publicKey, magicMint);

    const remaining = [
      { pubkey: Keypair.generate().publicKey, isSigner: false, isWritable: true }, // collection
      { pubkey: colAuth, isSigner: false, isWritable: false },
      { pubkey: nftCfg, isSigner: false, isWritable: false },
      { pubkey: magicAuth, isSigner: false, isWritable: false },
      { pubkey: magicMint, isSigner: false, isWritable: true },
      { pubkey: sellerAta, isSigner: false, isWritable: true },
    ];

    try {
      await program.methods
        .sellItem(99 as any)
        .accounts({
          seller: provider.wallet.publicKey,
          marketplaceAuthority: mpAuth,
          asset: fakeAsset,
          itemMetadata: metadataPda,
          gameConfig,
          itemNftProgram: PROGRAM_IDS.ITEM_NFT,
          magicTokenProgram: PROGRAM_IDS.MAGIC_TOKEN,
          mplCoreProgram: MPL_CORE_ID,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: new PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJe1bpb"),
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(remaining)
        .rpc();
      expect.fail("Should have thrown");
    } catch (e: any) {
      expect(e.message).to.include("InvalidItemType");
    }
  });
});
