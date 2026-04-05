/** Shared PDA helpers and generated-account loaders for off-chain scripts/tests. */
import * as anchor from "@coral-xyz/anchor";
import { readFileSync } from "fs";
import path from "path";
import { PROGRAM_IDS } from "./programs";

/** String map matching the JSON structure emitted into `utils/accounts.json`. */
export type GenericAccountMap = Record<string, string>;

const ACCOUNTS_FILE = path.resolve(__dirname, "accounts.json");

const GAME_CONFIG_SEED = Buffer.from("game_config");
const PLAYER_SEED = Buffer.from("player");
const PROGRAM_AUTHORITY_SEED = Buffer.from("program_authority");
const ITEM_METADATA_SEED = Buffer.from("item_metadata");
const RESOURCE_MINT_SEED = Buffer.from("resource_mint");
const MAGIC_TOKEN_MINT_SEED = Buffer.from("magic_token_mint");

/** Reads the latest generated account snapshot written by bootstrap scripts. */
export const loadAccounts = (): GenericAccountMap => {
  return JSON.parse(readFileSync(ACCOUNTS_FILE, "utf8")) as GenericAccountMap;
};

/** Derives the canonical `GameConfig` PDA for the resource manager program. */
export const findGameConfigPda = (): [anchor.web3.PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [GAME_CONFIG_SEED],
    new anchor.web3.PublicKey(PROGRAM_IDS.resource_manager),
  );
};

/** Derives the signer PDA used by a program for CPI authorization. */
export const findProgramAuthorityPda = (
  programId: anchor.web3.PublicKey,
): [anchor.web3.PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [PROGRAM_AUTHORITY_SEED],
    programId,
  );
};

/** Derives the search-player PDA for a wallet owner. */
export const findPlayerPda = (
  owner: anchor.web3.PublicKey,
): [anchor.web3.PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [PLAYER_SEED, owner.toBuffer()],
    new anchor.web3.PublicKey(PROGRAM_IDS.search),
  );
};

/** Derives the canonical resource mint PDA for a resource index. */
export const findResourceMintPda = (
  resourceType: number,
): [anchor.web3.PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [RESOURCE_MINT_SEED, Buffer.from([resourceType])],
    new anchor.web3.PublicKey(PROGRAM_IDS.resource_manager),
  );
};

/** Derives the gameplay metadata PDA tracked by the `item_nft` program. */
export const findItemMetadataPda = (
  mint: anchor.web3.PublicKey,
): [anchor.web3.PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [ITEM_METADATA_SEED, mint.toBuffer()],
    new anchor.web3.PublicKey(PROGRAM_IDS.item_nft),
  );
};

/** Derives the default reward-token mint PDA owned by `magic_token`. */
export const findMagicTokenMintPda = (): [anchor.web3.PublicKey, number] => {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [MAGIC_TOKEN_MINT_SEED],
    new anchor.web3.PublicKey(PROGRAM_IDS.magic_token),
  );
};
