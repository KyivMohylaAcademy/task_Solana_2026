use anchor_lang::prelude::*;

/// Глобальна конфігурація програми MagicToken.
/// PDA seeds: [b"magic_token_config"]
#[account]
pub struct MagicTokenConfig {
    /// Адміністратор програми
    pub admin: Pubkey,
    /// Адреса мінту MagicToken (Token-2022)
    pub mint: Pubkey,
    /// Авторизована PDA програми marketplace для виклику mint_magic_token
    pub marketplace_authority: Pubkey,
    /// Bump seed для цього PDA
    pub bump: u8,
}

impl MagicTokenConfig {
    pub const SPACE: usize = 8  // discriminator
        + 32                    // admin
        + 32                    // mint
        + 32                    // marketplace_authority
        + 1;                    // bump
}

/// PDA-авторитет, що підписує операції мінтингу MagicToken.
/// PDA seeds: [b"mt_mint_authority"]
#[account]
pub struct MintAuthority {
    /// Bump seed
    pub bump: u8,
}

impl MintAuthority {
    pub const SPACE: usize = 8 + 1;
}
