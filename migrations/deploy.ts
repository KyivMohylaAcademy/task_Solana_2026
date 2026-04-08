// Anchor migration script for deploying programs to Solana Devnet
const anchor = require("@coral-xyz/anchor");

module.exports = async function (provider: any) {
  // Configure the client to use the provider
  anchor.setProvider(provider);

  console.log("Deploying Козацький бізнес programs...");
  console.log("Provider endpoint:", provider.connection.rpcEndpoint);
  console.log("Wallet:", provider.wallet.publicKey.toBase58());
};
