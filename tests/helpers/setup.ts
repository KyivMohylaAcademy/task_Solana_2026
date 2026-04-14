/**
 * Test setup helpers: bankrun context, program clients, and shared fixtures.
 */
import { BankrunProvider } from "anchor-bankrun";
import { startAnchor, ProgramTestContext, Clock } from "solana-bankrun";
import {
  Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";

// ── Seeds (mirroring Rust consts) ─────────────────────────────────────────────
export const SEEDS = {
  GAME_CONFIG:                 Buffer.from("config"),
  RESOURCE_MINT:               Buffer.from("resource_mint"),
  RESOURCE_AUTHORITY:          Buffer.from("resource_authority"),
  SEARCH_AUTHORITY:            Buffer.from("search_authority"),
  CRAFTING_AUTHORITY:          Buffer.from("crafting_authority"),
  MARKETPLACE_AUTHORITY:       Buffer.from("marketplace_authority"),
  MAGIC_MINT:                  Buffer.from("magic_mint"),
  MAGIC_AUTHORITY:             Buffer.from("magic_authority"),
  MAGIC_CONFIG:                Buffer.from("magic_config"),
  PLAYER:                      Buffer.from("player"),
  ITEM:                        Buffer.from("item"),
  ITEM_COLLECTION_AUTHORITY:   Buffer.from("item_collection_authority"),
  ITEM_COLLECTION:             Buffer.from("item_collection"),
  ITEM_NFT_CONFIG:             Buffer.from("item_nft_config"),
};

// ── Program IDs (from Anchor.toml) ────────────────────────────────────────────
export const PROGRAM_IDS = {
  RESOURCE_MANAGER: new PublicKey("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"),
  MAGIC_TOKEN:      new PublicKey("3kbN2PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLn"),
  SEARCH:           new PublicKey("7bMD2PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLn"),
  ITEM_NFT:         new PublicKey("9cPR2PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLn"),
  CRAFTING:         new PublicKey("BdPR2PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLn"),
  MARKETPLACE:      new PublicKey("CePR2PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLn"),
};

// ── PDA helpers ───────────────────────────────────────────────────────────────
export function gameConfigPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEEDS.GAME_CONFIG], PROGRAM_IDS.RESOURCE_MANAGER);
}

export function resourceAuthorityPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEEDS.RESOURCE_AUTHORITY], PROGRAM_IDS.RESOURCE_MANAGER);
}

export function resourceMintPda(kind: number): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEEDS.RESOURCE_MINT, Buffer.from([kind])],
    PROGRAM_IDS.RESOURCE_MANAGER
  );
}

export function searchAuthorityPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEEDS.SEARCH_AUTHORITY], PROGRAM_IDS.SEARCH);
}

export function craftingAuthorityPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEEDS.CRAFTING_AUTHORITY], PROGRAM_IDS.CRAFTING);
}

export function marketplaceAuthorityPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEEDS.MARKETPLACE_AUTHORITY], PROGRAM_IDS.MARKETPLACE);
}

export function magicMintPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEEDS.MAGIC_MINT], PROGRAM_IDS.MAGIC_TOKEN);
}

export function magicAuthorityPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEEDS.MAGIC_AUTHORITY], PROGRAM_IDS.MAGIC_TOKEN);
}

export function magicConfigPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEEDS.MAGIC_CONFIG], PROGRAM_IDS.MAGIC_TOKEN);
}

export function playerPda(owner: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEEDS.PLAYER, owner.toBuffer()],
    PROGRAM_IDS.SEARCH
  );
}

export function itemMetadataPda(asset: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEEDS.ITEM, asset.toBuffer()],
    PROGRAM_IDS.ITEM_NFT
  );
}

export function itemNftConfigPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEEDS.ITEM_NFT_CONFIG], PROGRAM_IDS.ITEM_NFT);
}

export function collectionAuthorityPda(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([SEEDS.ITEM_COLLECTION_AUTHORITY], PROGRAM_IDS.ITEM_NFT);
}

// ── Clock helpers (bankrun) ───────────────────────────────────────────────────
export async function advanceClock(
  context: ProgramTestContext,
  addSeconds: bigint
): Promise<void> {
  const current = await context.banksClient.getClock();
  await context.setClock(
    new Clock(
      current.slot,
      current.epochStartTimestamp,
      current.epoch,
      current.leaderScheduleEpoch,
      current.unixTimestamp + addSeconds
    )
  );
}

// ── ATA helpers ───────────────────────────────────────────────────────────────
export function getResourceAta(owner: PublicKey, mint: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(mint, owner, false, TOKEN_2022_PROGRAM_ID);
}

export function getMagicAta(owner: PublicKey, magicMint: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(magicMint, owner, false, TOKEN_2022_PROGRAM_ID);
}

// ── Airdrop helper ────────────────────────────────────────────────────────────
export async function airdrop(
  context: ProgramTestContext,
  to: PublicKey,
  sol: number = 100
): Promise<void> {
  await context.banksClient.processTransaction(
    new anchor.web3.Transaction().add(
      SystemProgram.transfer({
        fromPubkey: context.payer.publicKey,
        toPubkey: to,
        lamports: sol * LAMPORTS_PER_SOL,
      })
    )
  );
}
