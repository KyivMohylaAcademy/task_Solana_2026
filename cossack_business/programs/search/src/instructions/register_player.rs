use anchor_lang::prelude::*;
use crate::state::Player;

/// Accounts for registering a new player.
#[derive(Accounts)]
pub struct RegisterPlayer<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        init,
        payer = player,
        space = 8 + Player::INIT_SPACE,
        seeds = [b"player", player.key().as_ref()],
        bump,
    )]
    pub player_account: Account<'info, Player>,

    pub system_program: Program<'info, System>,
}
