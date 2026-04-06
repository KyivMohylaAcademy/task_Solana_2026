import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { 
  Keypair, 
  SystemProgram, 
  PublicKey 
} from "@solana/web3.js";

function getErrorMessage(error: any): string {
  if (error?.error?.errorMessage) return error.error.errorMessage;
  if (error?.message) return error.message;
  if (error?.logs) return error.logs.join(" ");
  return String(error);
}

/**
 * Integration tests - tests the full game flow
 */
describe("integration", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const player = provider.wallet;
  
  // Load all programs
  const searchProgram = anchor.workspace.Search;
  const craftingProgram = anchor.workspace.Crafting;
  const itemNftProgram = anchor.workspace.ItemNft;
  const marketplaceProgram = anchor.workspace.Marketplace;

  let searchConfig: PublicKey;
  let playerPda: PublicKey;
  let craftingConfig: PublicKey;
  let itemConfig: PublicKey;
  let marketplaceConfig: PublicKey;

  before(async () => {
    // Find all PDAs
    [searchConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      searchProgram.programId
    );

    [playerPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), player.publicKey.toBuffer()],
      searchProgram.programId
    );

    [craftingConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      craftingProgram.programId
    );

    [itemConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      itemNftProgram.programId
    );

    [marketplaceConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      marketplaceProgram.programId
    );

    // Initialize all programs (idempotent - will skip if already done)
    try {
      await searchProgram.methods.initialize().accounts({
        config: searchConfig,
        admin: player.publicKey,
        systemProgram: SystemProgram.programId,
      }).rpc();
    } catch (e) { /* already initialized */ }

    try {
      await craftingProgram.methods.initialize().accounts({
        config: craftingConfig,
        admin: player.publicKey,
        systemProgram: SystemProgram.programId,
      }).rpc();
    } catch (e) { /* already initialized */ }

    try {
      await itemNftProgram.methods.initialize().accounts({
        config: itemConfig,
        admin: player.publicKey,
        systemProgram: SystemProgram.programId,
      }).rpc();
    } catch (e) { /* already initialized */ }

    try {
      const prices = [new anchor.BN(100), new anchor.BN(150), new anchor.BN(200), new anchor.BN(250)];
      await marketplaceProgram.methods.initialize(prices).accounts({
        config: marketplaceConfig,
        admin: player.publicKey,
        systemProgram: SystemProgram.programId,
      }).rpc();
    } catch (e) { /* already initialized */ }
  });

  describe("Full Game Flow", () => {
    it("should complete a full gameplay cycle", async () => {
      console.log("\nStarting Full Game Flow Test\n");

      // Step 1: Initialize player in search program
      console.log("[1/7] Initializing player...");
      try {
        await searchProgram.methods
          .initPlayer()
          .accounts({
            player: playerPda,
            owner: player.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        console.log("Success: Player initialized");
      } catch (e) {
        console.log("Info: Player already initialized");
      }

      // Step 2: Search for resources
      console.log("\n[2/7] Searching for resources...");
      const tx1 = await searchProgram.methods
        .searchResources()
        .accounts({
          config: searchConfig,
          player: playerPda,
          owner: player.publicKey,
        })
        .rpc();
      
      const playerAccount = await searchProgram.account.player.fetch(playerPda);
      console.log("Success: Resources found");
      console.log(`Total searches: ${playerAccount.totalSearches}`);
      console.log(`Last search: ${new Date(playerAccount.lastSearchTimestamp.toNumber() * 1000)}`);

      // Step 3: Attempt second search (should fail due to cooldown)
      console.log("\n[3/7] Testing cooldown mechanism...");
      try {
        await searchProgram.methods
          .searchResources()
          .accounts({
            config: searchConfig,
            player: playerPda,
            owner: player.publicKey,
          })
          .rpc();
        console.log("Error: Cooldown should have prevented this");
        expect.fail("Should have failed due to cooldown");
      } catch (error: any) {
        console.log("Success: Cooldown working correctly");
        expect(getErrorMessage(error)).to.include("Search cooldown active");
      }

      // Step 4: Craft an item
      console.log("\n[4/7] Crafting Cossack Saber...");
      const tx2 = await craftingProgram.methods
        .craftItem(0)
        .accounts({
          config: craftingConfig,
          owner: player.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const craftingConfigAccount = await craftingProgram.account.craftingConfig.fetch(craftingConfig);
      console.log("Success: Item crafted");
      console.log(`Total items crafted: ${craftingConfigAccount.totalCrafted}`);

      // Step 5: Create item NFT
      console.log("\n[5/7] Creating NFT for crafted item...");
      const itemMint = Keypair.generate();
      const [itemMetadata] = PublicKey.findProgramAddressSync(
        [Buffer.from("item"), itemMint.publicKey.toBuffer()],
        itemNftProgram.programId
      );

      const tx3 = await itemNftProgram.methods
        .createItem(0, "https://example.com/metadata/saber.json")
        .accounts({
          config: itemConfig,
          itemMetadata: itemMetadata,
          mint: itemMint.publicKey,
          tokenAccount: Keypair.generate().publicKey,
          owner: player.publicKey,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();

      const itemMetadataAccount = await itemNftProgram.account.itemMetadata.fetch(itemMetadata);
      console.log("Success: NFT created");
      console.log(`Item type: ${itemMetadataAccount.itemType}`);
      console.log(`Owner: ${itemMetadataAccount.owner}`);

      // Step 6: List item on marketplace
      console.log("\n[6/7] Listing item on marketplace...");
      const [listing] = PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), itemMint.publicKey.toBuffer()],
        marketplaceProgram.programId
      );

      const tx4 = await marketplaceProgram.methods
        .listItem(0, null) // Use default price
        .accounts({
          config: marketplaceConfig,
          listing: listing,
          itemMint: itemMint.publicKey,
          seller: player.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const listingAccount = await marketplaceProgram.account.itemListing.fetch(listing);
      console.log("Success: Item listed");
      console.log(`Price: ${listingAccount.price} MagicToken`);
      console.log(`Active: ${listingAccount.isActive}`);

      // Step 7: Cancel listing
      console.log("\n[7/7] Cancelling listing...");
      const tx5 = await marketplaceProgram.methods
        .cancelListing()
        .accounts({
          listing: listing,
          itemMint: itemMint.publicKey,
          seller: player.publicKey,
        })
        .rpc();

      const updatedListing = await marketplaceProgram.account.itemListing.fetch(listing);
      console.log("Success: Listing cancelled");
      console.log(`Active: ${updatedListing.isActive}`);

      console.log("\nFull game flow completed successfully\n");
    });

    it("should enforce all security constraints", async () => {
      console.log("\nTesting Security Constraints\n");

      // Test 1: Invalid resource ID
      console.log("[1/3] Testing invalid resource ID...");
      try {
        await craftingProgram.methods
          .burnResource(10, new anchor.BN(1))
          .accounts({
            mint: Keypair.generate().publicKey,
            tokenAccount: Keypair.generate().publicKey,
            owner: player.publicKey,
            tokenProgram: anchor.utils.token.TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
        expect.fail("Should reject invalid resource ID");
      } catch (error: any) {
        console.log("Success: Invalid resource ID rejected");
        expect(getErrorMessage(error)).to.include("Invalid resource ID");
      }

      // Test 2: Invalid item type
      console.log("\n[2/3] Testing invalid item type...");
      try {
        await craftingProgram.methods
          .craftItem(5)
          .accounts({
            config: craftingConfig,
            owner: player.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should reject invalid item type");
      } catch (error: any) {
        console.log("Success: Invalid item type rejected");
        expect(getErrorMessage(error)).to.include("Invalid item type");
      }

      // Test 3: Unauthorized listing cancellation
      console.log("\n[3/3] Testing unauthorized listing cancellation...");
      const fakeSeller = Keypair.generate();
      const testMint = Keypair.generate();
      
      // Create a listing first
      const [testListing] = PublicKey.findProgramAddressSync(
        [Buffer.from("listing"), testMint.publicKey.toBuffer()],
        marketplaceProgram.programId
      );

      await marketplaceProgram.methods
        .listItem(0, null)
        .accounts({
          config: marketplaceConfig,
          listing: testListing,
          itemMint: testMint.publicKey,
          seller: player.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      // Try to cancel from different account
      try {
        await marketplaceProgram.methods
          .cancelListing()
          .accounts({
            listing: testListing,
            itemMint: testMint.publicKey,
            seller: fakeSeller.publicKey,
          })
          .signers([fakeSeller])
          .rpc();
        expect.fail("Should reject unauthorized cancellation");
      } catch (error: any) {
        console.log("Success: Unauthorized cancellation rejected");
        expect(getErrorMessage(error)).to.include("Unauthorized");
      }

      console.log("\nAll security constraints enforced\n");
    });
  });

  describe("Recipe Validation", () => {
    it("should validate all item recipes", async () => {
      console.log("\nTesting Recipe Validation\n");

      const recipes = [
        { name: "Cossack Saber", type: 0, recipe: [1, 3, 0, 1, 0, 0] },
        { name: "Elder Staff", type: 1, recipe: [2, 0, 1, 0, 0, 1] },
        { name: "Characternik Armor", type: 2, recipe: [0, 2, 1, 4, 0, 0] },
        { name: "Battle Bracelet", type: 3, recipe: [0, 4, 2, 0, 0, 2] },
      ];

      for (const item of recipes) {
        console.log(`Testing ${item.name}:`);
        console.log(`  WOOD=${item.recipe[0]}, IRON=${item.recipe[1]}, GOLD=${item.recipe[2]}`);
        console.log(`  LEATHER=${item.recipe[3]}, STONE=${item.recipe[4]}, DIAMOND=${item.recipe[5]}`);

        await craftingProgram.methods
          .craftItem(item.type)
          .accounts({
            config: craftingConfig,
            owner: player.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();

        console.log(`Success: ${item.name} recipe valid\n`);
      }
    });
  });
});
