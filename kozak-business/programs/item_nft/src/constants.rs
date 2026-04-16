use anchor_lang::prelude::*;

/// PDA seed for the singleton [`crate::state::ItemConfig`] account.
#[constant]
pub const ITEM_CONFIG_SEED: &[u8] = b"item_config";

/// PDA seed for the program-wide mint-authority used for all NFT mints.
/// This program signs as this PDA when creating metadata, minting, and
/// calling CreateMasterEdition. Using one shared authority keeps the
/// accounts struct manageable; security is enforced by the Token program
/// (only the mint authority can mint) and by Metaplex (after
/// CreateMasterEdition the mint authority is frozen to the edition PDA).
#[constant]
pub const NFT_MINT_AUTHORITY_SEED: &[u8] = b"nft_mint_authority";

/// PDA seed the registered `marketplace` program must sign as when calling
/// the gated [`crate::instructions::burn_item_nft`] instruction. Mirrors the
/// `SEARCH_AUTHORITY_SEED` pattern used in the `search` ↔ `resource_manager`
/// interface.
#[constant]
pub const MARKETPLACE_AUTHORITY_SEED: &[u8] = b"marketplace_auth";
