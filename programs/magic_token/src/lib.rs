use anchor_lang::prelude::*;

declare_id!("GRdyjoQis5Pc7hfiHff8zNWhoZgvrYapcwgJdqFoWQQc");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

#[program]
pub mod magic_token {
    use super::*;

    /// Ініціалізує MagicToken SPL Token-2022 мінт.
    /// mint_authority встановлюється на Marketplace PDA.
    pub fn initialize_magic_token(ctx: Context<InitializeMagicToken>) -> Result<()> {
        instructions::initialize_magic_token::handler(ctx)
    }

    /// Мінтить MagicToken на акаунт отримувача. Тільки через CPI від marketplace.
    pub fn mint_magic_token(ctx: Context<MintMagicToken>, amount: u64) -> Result<()> {
        instructions::mint_magic_token::handler(ctx, amount)
    }
}
