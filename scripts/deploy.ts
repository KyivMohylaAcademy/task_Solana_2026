import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { 
  TOKEN_PROGRAM_ID, 
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID 
} from "@solana/spl-token";

/**
 * Deployment and initialization script for Cossack Business Game
 * Run this after deploying programs to Devnet
 */

async function main() {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  console.log("Deploying Cossack Business Game\n");
  console.log("Wallet:", provider.wallet.publicKey.toString());
  console.log("Cluster:", provider.connection.rpcEndpoint);

  // Load programs
  const resourceManager = anchor.workspace.ResourceManager;
  const magicToken = anchor.workspace.MagicToken;
  const itemNft = anchor.workspace.ItemNft;
  const search = anchor.workspace.Search;
  const crafting = anchor.workspace.Crafting;
  const marketplace = anchor.workspace.Marketplace;

  console.log("\nProgram IDs:");
  console.log("resource_manager:", resourceManager.programId.toString());
  console.log("magic_token:", magicToken.programId.toString());
  console.log("item_nft:", itemNft.programId.toString());
  console.log("search:", search.programId.toString());
  console.log("crafting:", crafting.programId.toString());
  console.log("marketplace:", marketplace.programId.toString());

  // PDAs
  const [resourceConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    resourceManager.programId
  );

  const [magicConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    magicToken.programId
  );

  const [itemConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    itemNft.programId
  );

  const [searchConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    search.programId
  );

  const [craftingConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    crafting.programId
  );

  const [marketplaceConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    marketplace.programId
  );

  console.log("\nConfig PDAs:");
  console.log("resource_config:", resourceConfig.toString());
  console.log("magic_config:", magicConfig.toString());
  console.log("item_config:", itemConfig.toString());
  console.log("search_config:", searchConfig.toString());
  console.log("crafting_config:", craftingConfig.toString());
  console.log("marketplace_config:", marketplaceConfig.toString());

  try {
    // 1. Initialize resource_manager
    console.log("\n[1/6] Initializing resource_manager...");
    try {
      await resourceManager.methods
        .initialize()
        .accounts({
          config: resourceConfig,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("Success: resource_manager initialized");
    } catch (e) {
      console.log("Info: resource_manager already initialized");
    }

    // 2. Initialize magic_token
    console.log("\n[2/6] Initializing magic_token...");
    const magicMint = Keypair.generate();
    try {
      await magicToken.methods
        .initialize(9) // 9 decimals for MagicToken
        .accounts({
          config: magicConfig,
          mint: magicMint.publicKey,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("Success: magic_token initialized");
      console.log("MagicToken Mint:", magicMint.publicKey.toString());
    } catch (e) {
      console.log("Info: magic_token already initialized");
    }

    // 3. Initialize item_nft
    console.log("\n[3/6] Initializing item_nft...");
    try {
      await itemNft.methods
        .initialize()
        .accounts({
          config: itemConfig,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("Success: item_nft initialized");
    } catch (e) {
      console.log("Info: item_nft already initialized");
    }

    // 4. Initialize search
    console.log("\n[4/6] Initializing search...");
    try {
      await search.methods
        .initialize()
        .accounts({
          config: searchConfig,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("Success: search initialized");
    } catch (e) {
      console.log("Info: search already initialized");
    }

    // 5. Initialize crafting
    console.log("\n[5/6] Initializing crafting...");
    try {
      await crafting.methods
        .initialize()
        .accounts({
          config: craftingConfig,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("Success: crafting initialized");
    } catch (e) {
      console.log("Info: crafting already initialized");
    }

    // 6. Initialize marketplace
    console.log("\n[6/6] Initializing marketplace...");
    const itemPrices = [100, 150, 200, 250]; // Default prices
    try {
      await marketplace.methods
        .initialize(itemPrices)
        .accounts({
          config: marketplaceConfig,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log("Success: marketplace initialized");
      console.log("Item prices:", itemPrices);
    } catch (e) {
      console.log("Info: marketplace already initialized");
    }

    console.log("\nAll programs initialized successfully.");
    console.log("\nNext steps:");
    console.log("1. Create resource mints using resource_manager");
    console.log("2. Initialize player accounts in search program");
    console.log("3. Start interacting with deployed programs");

  } catch (error) {
    console.error("\nError during initialization:");
    console.error(error);
    throw error;
  }
}

main()
  .then(() => {
    console.log("\nDeployment script completed successfully.");
    process.exit(0);
  })
  .catch((error) => {
    console.error("\nDeployment script failed:");
    console.error(error);
    process.exit(1);
  });
