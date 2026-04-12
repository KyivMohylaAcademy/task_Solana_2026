use anchor_lang::prelude::*;

declare_id!("6oHg42ymyViYzXQh5jfD13kNBEUPWeE73cSZ3buMGjsh");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

#[program]
pub mod resource_manager {
    use super::*;

    /// Ініціалізує GameConfig PDA та 6 SPL Token-2022 мінтів ресурсів.
    /// Може викликати тільки адмін. mint_authority = GameConfig PDA.
    pub fn initialize_game_config(ctx: Context<InitializeGameConfig>) -> Result<()> {
        instructions::initialize_game_config::handler(ctx)
    }

    /// Мінтить ресурс на акаунт гравця. Тільки через CPI від search/crafting.
    pub fn mint_resource(
        ctx: Context<MintResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        instructions::mint_resource::handler(ctx, resource_id, amount)
    }

    /// Спалює ресурс з акаунту гравця. Тільки через CPI від crafting.
    pub fn burn_resource(
        ctx: Context<BurnResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        instructions::burn_resource::handler(ctx, resource_id, amount)
    }
}
