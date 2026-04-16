use anchor_lang::prelude::*;

/// PDA seed for the singleton [`crate::state::GameConfig`] account.
#[constant]
pub const GAME_CONFIG_SEED: &[u8] = b"game_config";

/// PDA seed prefix for each resource's mint account. The full seeds are
/// `[RESOURCE_MINT_SEED, &[resource_id]]`, giving one deterministic mint
/// address per resource.
#[constant]
pub const RESOURCE_MINT_SEED: &[u8] = b"resource_mint";

/// PDA seed prefix for each resource's mint-authority. The authority is a
/// PDA with no private key — only this program's code can sign as it, so
/// nothing but our logic can mint or change the mint.
#[constant]
pub const MINT_AUTHORITY_SEED: &[u8] = b"mint_authority";

/// Seeds of the PDA that the admin-registered `search` program must sign
/// as when CPI-calling [`crate::instructions::mint_resource`]. The search
/// program derives the same PDA inside its own program ID; we validate it
/// via `seeds::program = game_config.search_program`.
#[constant]
pub const SEARCH_AUTHORITY_SEED: &[u8] = b"search_auth";

/// Number of base resources in the game (WOOD, IRON, GOLD, LEATHER, STONE,
/// DIAMOND). Keep in sync with the `resource_mints` array length in
/// [`crate::state::GameConfig`].
pub const RESOURCE_COUNT: usize = 6;

/// Human-readable name for each resource, indexed by `resource_id`. Written
/// into the mint's TokenMetadata extension so wallets and explorers display
/// something nicer than a raw pubkey.
pub const RESOURCE_NAMES: [&str; RESOURCE_COUNT] =
    ["Wood", "Iron", "Gold", "Leather", "Stone", "Diamond"];

/// Ticker-style symbol for each resource, same indexing as [`RESOURCE_NAMES`].
pub const RESOURCE_SYMBOLS: [&str; RESOURCE_COUNT] =
    ["WOOD", "IRON", "GOLD", "LTHR", "STNE", "DMND"];
