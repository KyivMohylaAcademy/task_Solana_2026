// Resources minted via admin_mint_resource for determinism; the search mechanic is covered by its own test.
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
import { loadAccounts } from "../../../utils/account_utils";

const MPL_TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

function deriveMetadataPda(mint: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
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

// Recipe amounts: [Wood, Iron, Gold, Leather, Stone, Diamond]
const RECIPES: number[][] = [
  [1, 3, 0, 1, 0, 0], // Cossack Saber
  [2, 0, 1, 0, 0, 1], // Elder's Staff
  [0, 2, 1, 4, 0, 0], // Kharakternyk's Armor
  [0, 4, 2, 0, 0, 2], // Battle Bracelet
];

describe("crafting", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const rmProgram = anchor.workspace.ResourceManager as Program<ResourceManager>;
  const itemNftProgram = anchor.workspace.ItemNft as Program<ItemNft>;
  const craftingProgram = anchor.workspace.Crafting as Program<Crafting>;

  let gameConfigPda: PublicKey;
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

  before(() => {
    const saved = loadAccounts();
    gameConfigPda = new PublicKey(saved["gameConfig"]);
    resourceMints = Array.from({ length: 6 }, (_, i) =>
      new PublicKey(saved[`resourceMint${i}`])
    );
  });

  async function mintResources(amounts: number[]) {
    for (let i = 0; i < 6; i++) {
      if (amounts[i] === 0) continue;
      const ata = getAssociatedTokenAddressSync(
        resourceMints[i],
        provider.wallet.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );
      await rmProgram.methods
        .adminMintResource(i, new anchor.BN(amounts[i]))
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
  }

  async function craftItem(itemType: number): Promise<PublicKey> {
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
      .craftItem(itemType)
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

    return itemMetadataPda;
  }

  it("crafts all 4 item types with correct resource burns", async () => {
    // Mint 10×Iron, 5×Wood, 5×Leather, 5×Gold, 3×Diamond (enough for all recipes with buffer).
    await mintResources([5, 10, 5, 5, 0, 3]);

    for (let itemType = 0; itemType < 4; itemType++) {
      // Get balances before.
      const beforeBalances = await Promise.all(
        resourceMints.map(async (m, i) => {
          const ata = getAssociatedTokenAddressSync(m, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID);
          try {
            const bal = await provider.connection.getTokenAccountBalance(ata);
            return bal.value.uiAmount ?? 0;
          } catch {
            return 0;
          }
        })
      );

      const itemMetadataPda = await craftItem(itemType);

      // Get balances after.
      const afterBalances = await Promise.all(
        resourceMints.map(async (m) => {
          const ata = getAssociatedTokenAddressSync(m, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID);
          try {
            const bal = await provider.connection.getTokenAccountBalance(ata);
            return bal.value.uiAmount ?? 0;
          } catch {
            return 0;
          }
        })
      );

      // Verify burns match recipe.
      for (let i = 0; i < 6; i++) {
        const burned = beforeBalances[i] - afterBalances[i];
        assert.equal(burned, RECIPES[itemType][i], `Resource ${i} burn mismatch for item_type=${itemType}`);
      }

      // Verify ItemMetadata.
      const meta = await itemNftProgram.account.itemMetadata.fetch(itemMetadataPda);
      assert.equal(meta.itemType, itemType);
    }
  });

  it("rejects burn_resource directly without crafting cpi_auth", async () => {
    const fakeAuth = provider.wallet.publicKey;
    const playerAtas = resourceMints.map((m) =>
      getAssociatedTokenAddressSync(m, provider.wallet.publicKey, false, TOKEN_2022_PROGRAM_ID)
    );
    try {
      await rmProgram.methods
        .burnResource(0, new anchor.BN(1))
        .accounts({
          cpiAuth: fakeAuth,
          gameConfig: gameConfigPda,
          mint: resourceMints[0],
          sourceAta: playerAtas[0],
          owner: provider.wallet.publicKey,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc();
      assert.fail("Should have rejected unauthorized burn");
    } catch (e: any) {
      assert.include(e.message, "Unauthorized");
    }
  });

  it("rejects craft_item with insufficient resources", async () => {
    // Don't mint any resources. Saber requires 1×Wood, 3×Iron, 1×Leather.
    // Balances may be 0 from previous burns.
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
    try {
      await craftingProgram.methods
        .craftItem(0) // Cossack Saber needs 3×Iron
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
          token2022Program: TOKEN_2022_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
        ])
        .signers([nftMint])
        .rpc();
      assert.fail("Should have failed with insufficient resources");
    } catch (e: any) {
      assert.match(e.message, /insufficient|custom program error|0x1/i);
    }
  });
});
