use anchor_lang::prelude::*;

/// MagicToken конфіг PDA. Seeds: ["magic_config"].
#[account]
pub struct MagicConfig {
    /// Адреса MagicToken мінту.
    pub mint: Pubkey,
    /// Marketplace PDA — єдиний авторизований mint_authority.
    pub marketplace_authority: Pubkey,
    /// PDA bump.
    pub bump: u8,
}

impl MagicConfig {
    pub const LEN: usize = 8 + 32 + 32 + 1;
}
