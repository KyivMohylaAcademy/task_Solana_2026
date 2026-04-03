/// Deployment utilities for Solana smart contracts

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

/**
 * Helper to find PDA addresses used in the programs
 */
export const findPDAs = (programId: PublicKey) => ({
  gameConfig: () => PublicKey.findProgramAddressSync(
    [Buffer.from("game_config")],
    programId
  ),
  playerSearch: (owner: PublicKey) => PublicKey.findProgramAddressSync(
    [Buffer.from("player_search"), owner.toBuffer()],
    programId
  ),
  itemMetadata: (itemMint: PublicKey) => PublicKey.findProgramAddressSync(
    [Buffer.from("item_metadata"), itemMint.toBuffer()],
    programId
  ),
  listing: (itemMint: PublicKey) => PublicKey.findProgramAddressSync(
    [Buffer.from("listing"), itemMint.toBuffer()],
    programId
  ),
  itemNFT: (itemMint: PublicKey) => PublicKey.findProgramAddressSync(
    [Buffer.from("item_nft"), itemMint.toBuffer()],
    programId
  ),
  magicTokenConfig: () => PublicKey.findProgramAddressSync(
    [Buffer.from("magic_token_config")],
    programId
  ),
});

/**
 * Crafting recipe definitions
 */
export const CRAFTING_RECIPES = [
  {
    id: 0,
    name: "Козацька шабля",
    resources: [1, 3, 0, 1, 0, 0], // Wood, Iron, Gold, Leather, Stone, Diamond
  },
  {
    id: 1,
    name: "Посох старійшини",
    resources: [2, 0, 1, 0, 0, 1],
  },
  {
    id: 2,
    name: "Броня характерника",
    resources: [0, 2, 1, 4, 0, 0],
  },
  {
    id: 3,
    name: "Бойовий браслет",
    resources: [0, 4, 2, 0, 0, 2],
  },
];

/**
 * Resource definitions
 */
export const RESOURCES = [
  { id: 0, name: "Дерево", symbol: "WOOD" },
  { id: 1, name: "Залізо", symbol: "IRON" },
  { id: 2, name: "Золото", symbol: "GOLD" },
  { id: 3, name: "Шкіра", symbol: "LEATHER" },
  { id: 4, name: "Камінь", symbol: "STONE" },
  { id: 5, name: "Алмаз", symbol: "DIAMOND" },
];

/**
 * Game configuration
 */
export const GAME_CONFIG = {
  SEARCH_INTERVAL: 60, // 60 seconds
  ITEM_DECIMALS: 0,
  RESOURCE_DECIMALS: 0,
};

/**
 * Wait for transaction confirmation
 */
export const waitForTransaction = async (
  connection: anchor.web3.Connection,
  signature: string,
  options?: { timeoutMs?: number }
) => {
  const timeoutMs = options?.timeoutMs || 30000;
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    const response = await connection.getSignatureStatus(signature);
    if (response.value?.confirmationStatus === "confirmed" ||
        response.value?.confirmationStatus === "finalized") {
      return true;
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }

  return false;
};
