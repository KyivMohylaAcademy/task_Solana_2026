use anchor_lang::prelude::*;

/// Marketplace listing for an item
#[account]
pub struct Listing {
    pub seller: Pubkey,
    pub item_mint: Pubkey,
    pub price: u64,
    pub bump: u8,
}

impl Listing {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 1;
}

/// Game configuration for marketplace
#[account]
pub struct MarketplaceConfig {
    pub admin: Pubkey,
    pub magic_token_mint: Pubkey,
    pub item_prices: [u64; 4], // prices for each item type
    pub bump: u8,
}

impl MarketplaceConfig {
    pub const SPACE: usize = 8 + 32 + 32 + (8 * 4) + 1;
}
