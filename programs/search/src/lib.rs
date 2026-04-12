use anchor_lang::prelude::*;

declare_id!("3BRTKJk5xnswDXDMKogS43hcJVdpr1XDJrStZyLGduvs");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

#[program]
pub mod search {
    use super::*;

    /// Ініціалізує Player PDA для гравця.
    pub fn initialize_player(ctx: Context<InitializePlayer>) -> Result<()> {
        instructions::initialize_player::handler(ctx)
    }

    /// Шукає 3 випадкових ресурси. Cooldown 60 секунд (он-чейн таймер).
    /// Псевдо-рандом через hash(timestamp, slot, owner). CPI → resource_manager.
    pub fn search_resources<'info>(ctx: Context<'info, SearchResources<'info>>) -> Result<()> {
        instructions::search_resources::handler(ctx)
    }
}
