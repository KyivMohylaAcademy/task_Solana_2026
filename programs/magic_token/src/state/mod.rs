use anchor_lang::prelude::*;

/// Magic token configuration
#[account]
pub struct MagicTokenConfig {
    pub mint: Pubkey,
    pub marketplace_program: Pubkey,
    pub bump: u8,
}

impl MagicTokenConfig {
    pub const SPACE: usize = 8 + 32 + 32 + 1;
}
