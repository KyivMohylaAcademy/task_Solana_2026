pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8KkKfeEqviwfGGnz1jLF5DVvhjC8HuSoXxnWJk4MQeoj");

/// Програма для керування MagicToken SPL Token-2022 мінтом гри "Козацький бізнес".
///
/// Забезпечує:
/// - Ініціалізацію конфігурації та PDA-авторитету мінтингу
/// - Створення Token-2022 мінту для MagicToken (decimals=0)
/// - Мінтинг MagicToken (тільки через marketplace програму via CPI)
#[program]
pub mod magic_token {
    use super::*;

    /// Ініціалізує конфігурацію MagicToken та створює Token-2022 мінт.
    pub fn initialize(
        ctx: Context<Initialize>,
        marketplace_authority: Pubkey,
    ) -> Result<()> {
        initialize::handler(ctx, marketplace_authority)
    }

    /// Мінтить MagicToken на акаунт гравця.
    /// Може викликатись тільки програмою marketplace через CPI.
    pub fn mint_magic_token(ctx: Context<MintMagicToken>, amount: u64) -> Result<()> {
        mint_magic_token::handler(ctx, amount)
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{MagicTokenConfig, MintAuthority};

    /// Перевіряє правильність розрахунку розміру акаунта MagicTokenConfig
    #[test]
    fn test_magic_token_config_space() {
        // discriminator(8) + admin(32) + mint(32) + marketplace_authority(32) + bump(1) = 105
        assert_eq!(MagicTokenConfig::SPACE, 8 + 32 + 32 + 32 + 1);
    }

    /// Перевіряє правильність розрахунку розміру акаунта MintAuthority
    #[test]
    fn test_mint_authority_space() {
        // discriminator(8) + bump(1) = 9
        assert_eq!(MintAuthority::SPACE, 8 + 1);
    }

    /// Перевіряє правильність розміру базового мінт-акаунту Token-2022
    #[test]
    fn test_basic_mint_size() {
        // Base Token-2022 mint: 82 bytes + account type(1) + padding(2) = 85
        assert_eq!(crate::constants::BASIC_MINT_SIZE, 85);
    }
}
