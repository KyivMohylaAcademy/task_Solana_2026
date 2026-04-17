use anchor_lang::prelude::*;

/// Глобальна конфігурація гри, яка зберігає посилання на всі ресурсні мінти.
/// PDA seeds: [b"game_config"]
#[account]
pub struct GameConfig {
    /// Адміністратор гри
    pub admin: Pubkey,
    /// Адреси мінтів для 6 ресурсів (0=WOOD, 1=IRON, 2=GOLD, 3=LEATHER, 4=STONE, 5=DIAMOND)
    pub resource_mints: [Pubkey; 6],
    /// Адреса мінту MagicToken
    pub magic_token_mint: Pubkey,
    /// Ціни предметів у MagicToken (для marketplace)
    pub item_prices: [u64; 4],
    /// Авторизована PDA програми search для виклику mint_resource
    pub search_authority: Pubkey,
    /// Авторизована PDA програми crafting для виклику mint/burn resource
    pub crafting_authority: Pubkey,
    /// Bump seed для цього PDA
    pub bump: u8,
}

impl GameConfig {
    pub const SPACE: usize = 8   // discriminator
        + 32                     // admin
        + 32 * 6                 // resource_mints
        + 32                     // magic_token_mint
        + 8 * 4                  // item_prices
        + 32                     // search_authority
        + 32                     // crafting_authority
        + 1;                     // bump
}

/// PDA-авторитет, що підписує mint/burn операції над ресурсними токенами.
/// PDA seeds: [b"mint_authority"]
#[account]
pub struct MintAuthority {
    /// Bump seed
    pub bump: u8,
}

impl MintAuthority {
    pub const SPACE: usize = 8 + 1;
}