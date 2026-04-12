use anchor_lang::prelude::*;

use crate::state::Player;

pub const PLAYER_SEED: &[u8] = b"player";

#[derive(Accounts)]
pub struct InitializePlayer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Player PDA для цього гравця.
    #[account(
        init,
        payer = owner,
        space = Player::LEN,
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump,
    )]
    pub player: Account<'info, Player>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializePlayer>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    player.owner = ctx.accounts.owner.key();
    player.last_search_timestamp = 0; // дозволяємо перший пошук одразу
    player.bump = ctx.bumps.player;

    msg!("Player ініціалізовано: {}", player.owner);
    Ok(())
}
