use anchor_lang::prelude::*;
use crate::state::*;

/// Ініціалізує PDA акаунт гравця для системи пошуку ресурсів.
#[derive(Accounts)]
pub struct InitializePlayer<'info> {
    /// Акаунт гравця (PDA)
    #[account(
        init,
        payer = player_wallet,
        space = Player::SPACE,
        seeds = [b"player", player_wallet.key().as_ref()],
        bump,
    )]
    pub player: Account<'info, Player>,

    /// Гаманець гравця (платник)
    #[account(mut)]
    pub player_wallet: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializePlayer>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    player.owner = ctx.accounts.player_wallet.key();
    // Встановлюємо timestamp 0 — гравець може одразу починати пошук
    player.last_search_timestamp = 0;
    player.bump = ctx.bumps.player;

    msg!(
        "Гравця ініціалізовано: {}",
        ctx.accounts.player_wallet.key()
    );
    Ok(())
}
