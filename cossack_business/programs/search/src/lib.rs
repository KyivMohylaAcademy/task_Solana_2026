use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("8idBXvmxQEwn8BCVe5W8nzJqktRsgubP1eFUJ6XQLuRc");

#[program]
pub mod search {
    use super::*;

    /// Register a new player PDA for the signer.
    pub fn register_player(ctx: Context<RegisterPlayer>) -> Result<()> {
        instructions::register_player::handler(ctx)
    }

    /// Search for resources. Mints 3 random resource tokens to the player's ATAs.
    /// Enforces SEARCH_COOLDOWN_SECONDS between calls.
    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        instructions::search_resources::handler(ctx)
    }
}
