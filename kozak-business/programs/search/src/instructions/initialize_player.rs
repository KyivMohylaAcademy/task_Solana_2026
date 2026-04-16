use anchor_lang::prelude::*;

use crate::{constants::PLAYER_SEED, state::Player};

/// Accounts required by [`handler`]. The player PDA address is derived from
/// the wallet's pubkey, so each wallet has exactly one Player account and
/// the `init` constraint guarantees a single initialisation.
#[derive(Accounts)]
pub struct InitializePlayer<'info> {
    /// The new `Player` PDA. `init` enforces that the account does not
    /// already exist; `payer = wallet` makes the wallet fund its own rent.
    #[account(
        init,
        payer = wallet,
        space = 8 + Player::INIT_SPACE,
        seeds = [PLAYER_SEED, wallet.key().as_ref()],
        bump,
    )]
    pub player: Account<'info, Player>,

    /// The owning wallet. Both the rent-payer and the seed source.
    #[account(mut)]
    pub wallet: Signer<'info>,

    /// Required to create the new account via CPI.
    pub system_program: Program<'info, System>,
}

/// Create a Player PDA for the calling wallet. Idempotent at the address
/// level — calling twice fails because Anchor's `init` refuses an existing
/// account.
pub fn handler(ctx: Context<InitializePlayer>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    player.wallet = ctx.accounts.wallet.key();
    player.last_search_timestamp = 0;
    player.bump = ctx.bumps.player;

    msg!("Player initialised for wallet {}", player.wallet);
    Ok(())
}
