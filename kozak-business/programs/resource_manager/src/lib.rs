//! `resource_manager` — owns the 6 in-game resource mints (SPL Token-2022)
//! and the global `GameConfig` account. All other programs CPI into this one
//! to mint/burn resources; direct Token Program calls are blocked by each
//! mint's authority being a PDA owned by this program.

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("AC4HSs3SakEbMAqefhDamXebxdGi3ZMktRWfyrXg22TR");

#[program]
pub mod resource_manager {
    use super::*;

    /// Create the singleton `GameConfig` PDA. See
    /// [`instructions::initialize_game_config`] for full docs.
    pub fn initialize_game_config(ctx: Context<InitializeGameConfig>) -> Result<()> {
        instructions::initialize_game_config::handler(ctx)
    }

    /// Create the Token-2022 mint for a single resource and record its
    /// address in `GameConfig`. See [`instructions::initialize_resource_mint`].
    pub fn initialize_resource_mint(
        ctx: Context<InitializeResourceMint>,
        resource_id: u8,
    ) -> Result<()> {
        instructions::initialize_resource_mint::handler(ctx, resource_id)
    }

    /// Admin-only: register the deployed `search` program ID. Required before
    /// any [`mint_resource`] CPI can succeed. See
    /// [`instructions::set_search_program`].
    pub fn set_search_program(
        ctx: Context<SetSearchProgram>,
        search_program: Pubkey,
    ) -> Result<()> {
        instructions::set_search_program::handler(ctx, search_program)
    }

    /// CPI-only entrypoint: mint resource tokens. Callable only by the
    /// registered search program (and any future programs we explicitly
    /// authorise the same way). See [`instructions::mint_resource`].
    pub fn mint_resource(
        ctx: Context<MintResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        instructions::mint_resource::handler(ctx, resource_id, amount)
    }
}
