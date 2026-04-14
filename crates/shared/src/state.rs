//! On-chain account structs shared across programs.

use anchor_lang::prelude::*;

/// Root game configuration. Stored as PDA `["config"]` in resource_manager.
#[account]
pub struct GameConfig {
    /// Admin who can update prices and settings.
    pub admin: Pubkey,
    /// Mint addresses for the 6 resource tokens (indexed by ResourceKind).
    pub resource_mints: [Pubkey; 6],
    /// MagicToken mint address (set by magic_token::initialize).
    pub magic_token_mint: Pubkey,
    /// MagicToken price for each of the 4 item types (indexed by ItemKind).
    pub item_prices: [u64; 4],
    /// Search cooldown in seconds. Default: 60. Admin-mutable for testing convenience.
    pub cooldown_seconds: i64,
    /// PDA bump for canonical derivation.
    pub bump: u8,
}

impl GameConfig {
    /// Fixed serialized size (used in `#[account(space = ...)]`).
    pub const LEN: usize = 8     // discriminator
        + 32                     // admin
        + 32 * 6                 // resource_mints
        + 32                     // magic_token_mint
        + 8 * 4                  // item_prices
        + 8                      // cooldown_seconds
        + 1;                     // bump
}

/// Per-player state. PDA `["player", owner]` in search program.
#[account]
pub struct Player {
    /// Wallet that owns this player account.
    pub owner: Pubkey,
    /// Unix timestamp of the last successful search (0 = never searched).
    pub last_search_timestamp: i64,
    /// Monotonically-increasing nonce fed into the RNG to prevent replay.
    pub search_nonce: u64,
    /// PDA bump.
    pub bump: u8,
}

impl Player {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 1;
}

/// Metadata for a crafted item NFT. PDA `["item", mint]` in item_nft program.
#[account]
pub struct ItemMetadata {
    /// Item type (0–3 corresponding to ItemKind).
    pub item_type: u8,
    /// Current owner of the mpl-core asset.
    pub owner: Pubkey,
    /// mpl-core asset address (the "mint" equivalent).
    pub mint: Pubkey,
    /// PDA bump.
    pub bump: u8,
}

impl ItemMetadata {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 1;
}

/// Config stored in item_nft program recording the collection asset address.
#[account]
pub struct ItemNftConfig {
    pub admin: Pubkey,
    /// Address of the mpl-core collection asset.
    pub collection: Pubkey,
    pub bump: u8,
}

impl ItemNftConfig {
    pub const LEN: usize = 8 + 32 + 32 + 1;
}
