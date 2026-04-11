// The crafting test burns all accumulated resources, so we re-mint fresh amounts here
// (1×Wood + 3×Iron + 1×Leather for a Cossack Saber).
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  ComputeBudgetProgram,
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { assert } from "chai";
import { ResourceManager } from "../../../target/types/resource_manager";
import { ItemNft } from "../../../target/types/item_nft";
import { Crafting } from "../../../target/types/crafting";
import { Marketplace } from "../../../target/types/marketplace";
import { loadAccounts } from "../../../utils/account_utils";

const MPL_TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

function deriveMetadataPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    MPL_TOKEN_METADATA_PROGRAM_ID
  )[0];
}

function deriveMasterEditionPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from("edition"),
    ],
    MPL_TOKEN_METADATA_PROGRAM_ID
  )[0];
}

describe("marketplace", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const rmProgram = anchor.workspace.ResourceManager as Program<ResourceManager>;
  const itemNftProgram = anchor.workspace.ItemNft as Program<ItemNft>;
  const craftingProgram = anchor.workspace.Crafting as Program<Crafting>;
  const marketplaceProgram = anchor.workspace.Marketplace as Program<Marketplace>;

  let gameConfigPda: PublicKey;
  let magicTokenMint: PublicKey;
  let resourceMints: PublicKey[];

  const [resourceMintAuth] = PublicKey.findProgramAddressSync(
    [Buffer.from("resource_mint_auth")],
    new anchor.web3.PublicKey("DFtQE4puDvEMk1vYHhx3gQvfjUieWj1YtkhDKoyGCG1y")
  );
  const [nftAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("nft_authority")],
    new anchor.web3.PublicKey("2DqgLTXd1joDVbtu3DSbocd8C9zExybcdzYH7a6gUXno")
  );
  const [craftingCpiAuth] = PublicKey.findProgramAddressSync(
    [Buffer.from("cpi_auth")],
    new anchor.web3.PublicKey("YR3AszQR5gP98pMuzFb81Apb5KCsFi7U1gsSxfFeocF")
  );
  const [marketplaceCpiAuth] = PublicKey.findProgramAddressSync(
    [Buffer.from("cpi_auth")],
    new anchor.web3.PublicKey("6mYp9XMhdaqcRq9xh4EDBmRDGaDEEphzEJzpPF5KEpvX")
  );
  const [magicMintAuth] = PublicKey.findProgramAddressSync(
    [Buffer.from("magic_mint_auth")],
    new anchor.web3.PublicKey("5sk7gq8TwXpGFe7bxCsgWJ2k7StymKfXzkUD7HUfcMaY")
  );

  before(() => {
    const saved = loadAccounts();
    gameConfigPda = new PublicKey(saved["gameConfig"]);
    magicTokenMint = new PublicKey(saved["magicTokenMint"]);
    resourceMints = Array.from({ length: 6 }, (_, i) =>
      new PublicKey(saved[`resourceMint${i}`])
    );
  });

  it("crafts and sells Cossack Saber, receives 10 MagicToken", async () => {
    // Resource setup: mint Cossack Saber recipe (1×Wood + 3×Iron + 1×Leather).
    const saberAmounts = [1, 3, 0, 1, 0, 0];
    for (let i = 0; i < 6; i++) {
      if (saberAmounts[i] === 0) continue;
      const ata = getAssociatedTokenAddressSync(
        resourceMints[i],
        provider.wallet.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );
      await rmProgram.methods
        .adminMintResource(i, new anchor.BN(saberAmounts[i]))
        .accounts({
          admin: provider.wallet.publicKey,
          gameConfig: gameConfigPda,
          mint: resourceMints[i],
          recipientAta: ata,
          recipient: provider.wallet.publicKey,
          resourceMintAuth,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    }

    // Craft Cossack Saber.
    const nftMint = Keypair.generate();
    const recipientNftAta = getAssociatedTokenAddressSync(
      nftMint.publicKey,
      provider.wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const metadata = deriveMetadataPda(nftMint.publicKey);
    const masterEdition = deriveMasterEditionPda(nftMint.publicKey);
    const [itemMetadataPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("item_metadata"), nftMint.publicKey.toBuffer()],
      itemNftProgram.programId
    );
    const playerAtas = resourceMints.map((m) =>
      getAssociatedTokenAddressSync(m, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID)
    );

    await craftingProgram.methods
      .craftItem(0)
      .accounts({
        playerWallet: provider.wallet.publicKey,
        nftMint: nftMint.publicKey,
        gameConfig: gameConfigPda,
        cpiAuth: craftingCpiAuth,
        resourceMintAuth,
        mint0: resourceMints[0],
        mint1: resourceMints[1],
        mint2: resourceMints[2],
        mint3: resourceMints[3],
        mint4: resourceMints[4],
        mint5: resourceMints[5],
        ata0: playerAtas[0],
        ata1: playerAtas[1],
        ata2: playerAtas[2],
        ata3: playerAtas[3],
        ata4: playerAtas[4],
        ata5: playerAtas[5],
        nftAuthority,
        metadata,
        masterEdition,
        itemMetadata: itemMetadataPda,
        recipientNftAta,
        resourceManagerProgram: rmProgram.programId,
        itemNftProgram: itemNftProgram.programId,
        tokenMetadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
      ])
      .signers([nftMint])
      .rpc();

    // Seller's MagicToken ATA (may not exist yet — created by marketplace CPI).
    const sellerMagicAta = getAssociatedTokenAddressSync(
      magicTokenMint,
      provider.wallet.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    // Sell the NFT.
    await marketplaceProgram.methods
      .sellItem()
      .accounts({
        seller: provider.wallet.publicKey,
        gameConfig: gameConfigPda,
        nftMint: nftMint.publicKey,
        sellerNftAta: recipientNftAta,
        itemMetadata: itemMetadataPda,
        nftAuthority,
        cpiAuth: marketplaceCpiAuth,
        magicTokenMint,
        sellerMagicAta,
        magicMintAuth,
        itemNftProgram: itemNftProgram.programId,
        magicTokenProgram: new anchor.web3.PublicKey("5sk7gq8TwXpGFe7bxCsgWJ2k7StymKfXzkUD7HUfcMaY"),
        tokenProgram: TOKEN_PROGRAM_ID,
        token2022Program: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
      .rpc();

    // Verify NFT ATA is closed (account no longer exists on-chain).
    const nftAtaInfo = await provider.connection.getAccountInfo(recipientNftAta);
    assert.isNull(nftAtaInfo, "NFT ATA should be closed after sale");

    // Verify ItemMetadata PDA is closed.
    const itemMetaInfo = await provider.connection.getAccountInfo(itemMetadataPda);
    assert.isNull(itemMetaInfo, "ItemMetadata should be closed after sale");

    // Verify seller received 10 MagicToken (Cossack Saber price).
    const magicBalance = await provider.connection.getTokenAccountBalance(sellerMagicAta);
    assert.equal(magicBalance.value.uiAmount, 10, "Seller should receive 10 MagicToken");
  });

  it("rejects direct burn_nft without marketplace cpi_auth", async () => {
    // We need a fresh NFT to attempt burn on — but we don't have one here.
    // Instead just verify the guard by passing a fake cpi_auth.
    const fakeNftMint = Keypair.generate();
    const fakeAta = getAssociatedTokenAddressSync(
      fakeNftMint.publicKey,
      provider.wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const [fakeItemMeta] = PublicKey.findProgramAddressSync(
      [Buffer.from("item_metadata"), fakeNftMint.publicKey.toBuffer()],
      itemNftProgram.programId
    );

    try {
      await itemNftProgram.methods
        .burnNft()
        .accounts({
          payer: provider.wallet.publicKey,
          cpiAuth: provider.wallet.publicKey, // fake — not from marketplace
          holder: provider.wallet.publicKey,
          nftMint: fakeNftMint.publicKey,
          holderNftAta: fakeAta,
          itemMetadata: fakeItemMeta,
          nftAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should have rejected unauthorized burn_nft");
    } catch (e: any) {
      assert.include(e.message, "Unauthorized");
    }
  });

  it("rejects sell_item by wallet that doesn't hold the NFT", async () => {
    // Try to sell with NFT ATA balance 0 (or non-existent).
    // Create a fresh mint that was never given to anybody.
    const fakeNftMint = Keypair.generate();
    const fakeNftAta = getAssociatedTokenAddressSync(
      fakeNftMint.publicKey,
      provider.wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );
    const [fakeItemMeta] = PublicKey.findProgramAddressSync(
      [Buffer.from("item_metadata"), fakeNftMint.publicKey.toBuffer()],
      itemNftProgram.programId
    );
    const sellerMagicAta = getAssociatedTokenAddressSync(
      magicTokenMint,
      provider.wallet.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    try {
      await marketplaceProgram.methods
        .sellItem()
        .accounts({
          seller: provider.wallet.publicKey,
          gameConfig: gameConfigPda,
          nftMint: fakeNftMint.publicKey,
          sellerNftAta: fakeNftAta,
          itemMetadata: fakeItemMeta,
          nftAuthority,
          cpiAuth: marketplaceCpiAuth,
          magicTokenMint,
          sellerMagicAta,
          magicMintAuth,
          itemNftProgram: itemNftProgram.programId,
          magicTokenProgram: new anchor.web3.PublicKey("5sk7gq8TwXpGFe7bxCsgWJ2k7StymKfXzkUD7HUfcMaY"),
          tokenProgram: TOKEN_PROGRAM_ID,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
        .rpc();
      assert.fail("Should have rejected non-holder sell attempt");
    } catch (e: any) {
      assert.match(e.message, /NotNftHolder|AccountNotInitialized|not.*exist/i);
    }
  });
});
