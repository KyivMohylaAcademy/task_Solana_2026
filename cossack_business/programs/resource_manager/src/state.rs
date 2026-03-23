use anchor_lang::prelude::*;

/// Central game configuration PDA storing admin settings, authorized programs,
/// resource mints, item prices, rarity weights, and search cooldown.
#[account]
#[derive(InitSpace)]
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; 6],
    pub search_program: Pubkey,
    pub crafting_program: Pubkey,
    pub marketplace_program: Pubkey,
    pub item_prices: [u64; 4],
    /// Admin-configurable drop weights per resource (must sum to 100).
    pub rarity_weights: [u8; 6],
    /// Search cooldown in seconds (admin-configurable, default 60).
    pub search_cooldown: i64,
    pub resource_count: u8,
    pub bump: u8,
    pub mint_authority_bump: u8,
}
