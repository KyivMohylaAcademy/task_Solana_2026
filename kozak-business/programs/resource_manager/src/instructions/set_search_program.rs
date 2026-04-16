use anchor_lang::prelude::*;

use crate::{constants::GAME_CONFIG_SEED, state::GameConfig};

/// Accounts required by [`handler`]. Admin-gated via the `has_one = admin`
/// constraint on `GameConfig`.
#[derive(Accounts)]
pub struct SetSearchProgram<'info> {
    /// The `GameConfig` PDA we are mutating.
    #[account(
        mut,
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        has_one = admin,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// Must match `game_config.admin`. Doesn't need to be `mut` because we
    /// don't take any lamports — it's just here to authenticate the call.
    pub admin: Signer<'info>,
}

/// Register the address of the deployed `search` program with `GameConfig`.
///
/// `mint_resource` (the gated CPI entrypoint) checks that its `search_authority`
/// PDA is derived under exactly this program ID via Anchor's
/// `seeds::program = game_config.search_program` constraint. Without this
/// step, no CPI can mint resources.
pub fn handler(ctx: Context<SetSearchProgram>, search_program: Pubkey) -> Result<()> {
    ctx.accounts.game_config.search_program = search_program;
    msg!("Registered search program: {}", search_program);
    Ok(())
}
