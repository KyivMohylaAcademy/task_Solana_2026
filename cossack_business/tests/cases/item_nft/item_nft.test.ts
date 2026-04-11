// mint_nft requires crafting's cpi_auth, so the happy path is exercised via craft_item.
// Resource setup uses admin_mint_resource for the Cossack Saber recipe (1×Wood + 3×Iron + 1×Leather).
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ComputeBudgetProgram, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { assert } from "chai";
import { ResourceManager } from "../../../target/types/resource_manager";
import { ItemNft } from "../../../target/types/item_nft";
import { Crafting } from "../../../target/types/crafting";
import { loadAccounts } from "../../../utils/account_utils";

const MPL_TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

function deriveMetadataPda(mint: PublicKey): PublicKey {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    MPL_TOKEN_METADATA_PROGRAM_ID
  )[0];
}

function deriveMasterEditionPda(mint: PublicKey): PublicKey {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from("edition"),
    ],
    MPL_TOKEN_METADATA_PROGRAM_ID
  )[0];
}

describe("item_nft (via craft_item, cpi_auth guard active)", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const rmProgram = anchor.workspace.ResourceManager as Program<ResourceManager>;
  const itemNftProgram = anchor.workspace.ItemNft as Program<ItemNft>;
  const craftingProgram = anchor.workspace.Crafting as Program<Crafting>;

  let gameConfigPda: PublicKey;
  let resourceMints: PublicKey[];

  const [gameConfigPdaRm] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("game_config")],
    rmProgram.programId
  );
  const [resourceMintAuth] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("resource_mint_auth")],
    rmProgram.programId
  );
  const [nftAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("nft_authority")],
    itemNftProgram.programId
  );
  const [craftingCpiAuth] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("cpi_auth")],
    craftingProgram.programId
  );
  const [craftingResourceMintAuth] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("resource_mint_auth")],
    rmProgram.programId
  );

  before(() => {
    const saved = loadAccounts();
    gameConfigPda = new PublicKey(saved["gameConfig"]);
    resourceMints = Array.from({ length: 6 }, (_, i) =>
      new PublicKey(saved[`resourceMint${i}`])
    );
  });

  it("mints Cossack Saber (item_type=0) via craft_item", async () => {
    // Setup: admin mint 1×Wood + 3×Iron + 1×Leather for the recipe.
    const resourceAmounts = [1, 3, 0, 1, 0, 0]; // recipe amounts
    for (let i = 0; i < 6; i++) {
      if (resourceAmounts[i] === 0) continue;
      const ata = getAssociatedTokenAddressSync(
        resourceMints[i],
        provider.wallet.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );
      await rmProgram.methods
        .adminMintResource(i, new anchor.BN(resourceAmounts[i]))
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
    const [itemMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item_metadata"), nftMint.publicKey.toBuffer()],
      itemNftProgram.programId
    );

    const playerAtas = resourceMints.map((mint) =>
      getAssociatedTokenAddressSync(mint, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID)
    );

    const tx = await craftingProgram.methods
      .craftItem(0)
      .accounts({
        playerWallet: provider.wallet.publicKey,
        nftMint: nftMint.publicKey,
        gameConfig: gameConfigPda,
        cpiAuth: craftingCpiAuth,
        resourceMintAuth: craftingResourceMintAuth,
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

    console.log("craft_item tx:", tx);

    // Verify ItemMetadata PDA.
    const meta = await itemNftProgram.account.itemMetadata.fetch(itemMetadataPda);
    assert.equal(meta.itemType, 0, "item_type should be 0 (Cossack Saber)");

    // Verify NFT is in player's ATA.
    const ataBalance = await provider.connection.getTokenAccountBalance(recipientNftAta);
    assert.equal(ataBalance.value.uiAmount, 1, "Player should have 1 NFT");

    // Verify Metaplex metadata account exists.
    const metaInfo = await provider.connection.getAccountInfo(metadata);
    assert.isNotNull(metaInfo, "Metaplex metadata should exist");

    // Verify master edition exists.
    const editionInfo = await provider.connection.getAccountInfo(masterEdition);
    assert.isNotNull(editionInfo, "Master edition should exist");
  });

  it("rejects direct mint_nft without crafting cpi_auth", async () => {
    const nftMint = Keypair.generate();
    const metadata = deriveMetadataPda(nftMint.publicKey);
    const masterEdition = deriveMasterEditionPda(nftMint.publicKey);
    const [itemMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item_metadata"), nftMint.publicKey.toBuffer()],
      itemNftProgram.programId
    );
    const recipientNftAta = getAssociatedTokenAddressSync(
      nftMint.publicKey,
      provider.wallet.publicKey,
      false,
      TOKEN_PROGRAM_ID
    );

    // Use a fake cpi_auth (not from the crafting program).
    const fakeAuth = provider.wallet.publicKey;

    try {
      await itemNftProgram.methods
        .mintNft(0)
        .accounts({
          payer: provider.wallet.publicKey,
          recipient: provider.wallet.publicKey,
          cpiAuth: fakeAuth,
          nftMint: nftMint.publicKey,
          recipientNftAta,
          nftAuthority,
          metadata,
          masterEdition,
          itemMetadata: itemMetadataPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([nftMint])
        .rpc();
      assert.fail("Should have rejected unauthorized mint_nft");
    } catch (e: any) {
      assert.include(e.message, "Unauthorized");
    }
  });

  it("rejects craft_item with item_type=99", async () => {
    const nftMint = Keypair.generate();
    try {
      await craftingProgram.methods
        .craftItem(99)
        .accounts({
          playerWallet: provider.wallet.publicKey,
          nftMint: nftMint.publicKey,
          gameConfig: gameConfigPda,
          cpiAuth: craftingCpiAuth,
          resourceMintAuth: craftingResourceMintAuth,
          mint0: resourceMints[0],
          mint1: resourceMints[1],
          mint2: resourceMints[2],
          mint3: resourceMints[3],
          mint4: resourceMints[4],
          mint5: resourceMints[5],
          ata0: resourceMints[0], // placeholder
          ata1: resourceMints[1],
          ata2: resourceMints[2],
          ata3: resourceMints[3],
          ata4: resourceMints[4],
          ata5: resourceMints[5],
          nftAuthority,
          metadata: resourceMints[0],
          masterEdition: resourceMints[0],
          itemMetadata: resourceMints[0],
          recipientNftAta: resourceMints[0],
          resourceManagerProgram: rmProgram.programId,
          itemNftProgram: itemNftProgram.programId,
          token2022Program: TOKEN_2022_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 })])
        .signers([nftMint])
        .rpc();
      assert.fail("Should have rejected item_type=99");
    } catch (e: any) {
      assert.include(e.message, "InvalidItemType");
    }
  });
});
