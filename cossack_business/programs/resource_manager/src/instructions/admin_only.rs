use anchor_lang::prelude::*;
use crate::state::GameConfig;
use crate::errors::GameError;

/// Admin-only context for updating game configuration parameters.
#[derive(Accounts)]
pub struct AdminOnly<'info> {
    #[account(
        constraint = admin.key() == game_config.admin @ GameError::Unauthorized,
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump,
    )]
    pub game_config: Account<'info, GameConfig>,
}
