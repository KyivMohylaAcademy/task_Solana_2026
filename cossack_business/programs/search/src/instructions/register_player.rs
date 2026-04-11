use anchor_lang::prelude::*;
use crate::state::Player;

#[derive(Accounts)]
pub struct RegisterPlayer<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Player PDA — seeds use the signer's key so each wallet registers only its own player.
    #[account(
        init,
        payer = signer,
        space = Player::LEN,
        seeds = [b"player", signer.key().as_ref()],
        bump,
    )]
    pub player: Account<'info, Player>,

    pub system_program: Program<'info, System>,
}

/// Create the Player PDA for the signer. Sets last_search_timestamp = 0.
pub fn handler(ctx: Context<RegisterPlayer>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    player.owner = ctx.accounts.signer.key();
    player.last_search_timestamp = 0;
    player.bump = ctx.bumps.player;
    Ok(())
}
