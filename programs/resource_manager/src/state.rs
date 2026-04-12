use anchor_lang::prelude::*;

/// Центральний конфіг гри. PDA seeds: ["game_config"].
/// Зберігає адреси 6 ресурсних мінтів і MagicToken мінту.
#[account]
pub struct GameConfig {
    /// Адміністратор гри (може ініціалізувати config).
    pub admin: Pubkey,
    /// Масив мінт-адрес 6 ресурсів: [WOOD, IRON, GOLD, LEATHER, STONE, DIAMOND].
    pub resource_mints: [Pubkey; 6],
    /// Мінт адреса MagicToken.
    pub magic_token_mint: Pubkey,
    /// Ціни предметів у MagicToken: [item_0, item_1, item_2, item_3].
    pub item_prices: [u64; 4],
    /// PDA bump.
    pub bump: u8,
}

impl GameConfig {
    pub const LEN: usize = 8   // discriminator
        + 32                   // admin
        + 32 * 6               // resource_mints
        + 32                   // magic_token_mint
        + 8 * 4                // item_prices
        + 1;                   // bump
}
