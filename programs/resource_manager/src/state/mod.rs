use anchor_lang::prelude::*;

/// Game configuration storing resource mints and admin
#[account]
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; 6],
    pub bump: u8,
}

impl GameConfig {
    pub const SPACE: usize = 8 + 32 + (32 * 6) + 1;
}
