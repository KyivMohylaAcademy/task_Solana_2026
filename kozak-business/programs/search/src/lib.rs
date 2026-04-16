//! `search` — drives the resource-gathering loop. Players initialise a
//! Player PDA, then call `search_resources` (subject to a cooldown) to
//! receive a few pseudo-randomly drawn resource tokens minted via a gated
//! CPI into `resource_manager`.

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8DHJBMqyodTKcEaix734FAjsLRaMj2q1fnxSmaMVnUfV");

#[program]
pub mod search {
    use super::*;

    /// Create a `Player` PDA for the calling wallet. See
    /// [`instructions::initialize_player`].
    pub fn initialize_player(ctx: Context<InitializePlayer>) -> Result<()> {
        instructions::initialize_player::handler(ctx)
    }

    /// Run a search turn: cooldown-checked, pseudo-randomly mints
    /// `RESOURCES_PER_SEARCH` tokens via CPI into `resource_manager`. See
    /// [`instructions::search_resources`].
    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        instructions::search_resources::handler(ctx)
    }
}
