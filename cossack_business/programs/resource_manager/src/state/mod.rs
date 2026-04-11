use anchor_lang::prelude::*;

/// Global game configuration. Spec-compliant: exactly 5 fields + bump.
#[account]
pub struct GameConfig {
    /// The admin pubkey (deployer wallet).
    pub admin: Pubkey,
    /// The 6 resource mint addresses, indexed by resource ID (0–5).
    pub resource_mints: [Pubkey; 6],
    /// The MagicToken mint address.
    pub magic_token_mint: Pubkey,
    /// Prices in MagicToken for each of the 4 item types.
    pub item_prices: [u64; 4],
    /// PDA bump.
    pub bump: u8,
}

impl GameConfig {
    pub const LEN: usize = 8  // discriminator
        + 32              // admin
        + 32 * 6          // resource_mints
        + 32              // magic_token_mint
        + 8 * 4           // item_prices
        + 1;              // bump
}
