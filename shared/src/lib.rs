//! Shared account layouts and enums used across multiple programs in the workspace.

use anchor_lang::prelude::*;

declare_id!("HAktvQC29ctNNZ1YHv3HTqVLGxsWE7UYLJcXBAByVGwP");

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ItemType {
    Saber = 0,
    Staff = 1,
    Armor = 2,
    Bracelet = 3,
}

impl ItemType {
    /// Returns the number of distinct item types.
    pub const fn count() -> usize {
        4
    }
}

/// Global game configuration.
#[account]
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; 6],
    pub magic_token_mint: Pubkey,
    pub item_prices: [u64; 4],
    pub search_program: Pubkey,
    pub crafting_program: Pubkey,
    pub item_nft_program: Pubkey,
    pub marketplace_program: Pubkey,
    pub bump: u8,
}

impl GameConfig {
    pub const LEN: usize = 8 + 32 + (32 * 6) + 32 + (8 * 4) + (32 * 4) + 1;
}

/// Per-player state for search cooldown.
#[account]
pub struct Player {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub search_nonce: u64,
    pub bump: u8,
}

impl Player {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 1;
}

/// On-chain metadata for a crafted item NFT.
#[account]
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

impl ItemMetadata {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 1;
}

/// Active marketplace listing for an item mint.
#[account]
pub struct Listing {
    pub seller: Pubkey,
    pub item_mint: Pubkey,
    pub price: u64,
    pub bump: u8,
}

impl Listing {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 1;
}
