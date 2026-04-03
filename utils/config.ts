/// Configuration for different Solana clusters

export interface ClusterConfig {
  name: string;
  url: string;
  programIds: {
    resourceManager: string;
    search: string;
    crafting: string;
    itemNft: string;
    marketplace: string;
    magicToken: string;
  };
}

export const DEVNET_CONFIG: ClusterConfig = {
  name: "devnet",
  url: "https://api.devnet.solana.com",
  programIds: {
    resourceManager: "ResourceMgrXXXXXXXXXXXXXXXXXXXXXXXXXX",
    search: "SearchXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    crafting: "CraftingXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    itemNft: "ItemNFTXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    marketplace: "MarketplaceXXXXXXXXXXXXXXXXXXXXXXXX",
    magicToken: "MagicTokenXXXXXXXXXXXXXXXXXXXXXXXX",
  },
};

export const LOCALHOST_CONFIG: ClusterConfig = {
  name: "localhost",
  url: "http://localhost:8899",
  programIds: {
    resourceManager: "ResourceMgrXXXXXXXXXXXXXXXXXXXXXXXXXX",
    search: "SearchXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    crafting: "CraftingXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    itemNft: "ItemNFTXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
    marketplace: "MarketplaceXXXXXXXXXXXXXXXXXXXXXXXX",
    magicToken: "MagicTokenXXXXXXXXXXXXXXXXXXXXXXXX",
  },
};

export const getConfig = (network: string): ClusterConfig => {
  switch (network) {
    case "devnet":
      return DEVNET_CONFIG;
    case "localhost":
      return LOCALHOST_CONFIG;
    default:
      return DEVNET_CONFIG;
  }
};
