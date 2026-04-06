// Anchor migration file
// This is run when you execute `anchor migrate`

const anchor = require("@coral-xyz/anchor");

module.exports = async function (provider) {
  // Configure client to use the provider
  anchor.setProvider(provider);

  console.log("Running migrations...");
  console.log("Wallet:", provider.wallet.publicKey.toString());
  
  // Add your deployment logic here
  // This can include:
  // - Initializing program accounts
  // - Setting up initial state
  // - Creating resource mints
  // etc.
  
  console.log("Migration complete!");
};
