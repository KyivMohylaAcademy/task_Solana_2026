use anchor_lang::prelude::*;

use crate::{constants::GAME_CONFIG_SEED, state::GameConfig};

/// Accounts required by [`handler`]. Every constraint on every field is
/// validated by Anchor *before* `handler` runs — if any check fails the
/// transaction aborts with no state change.
#[derive(Accounts)]
pub struct InitializeGameConfig<'info> {
    /// The `GameConfig` PDA we are creating.
    ///
    /// - `init` — this account must not exist yet; Anchor creates it.
    /// - `payer = admin` — the admin funds the account's rent.
    /// - `space = 8 + GameConfig::INIT_SPACE` — 8-byte Anchor discriminator
    ///   plus the serialized struct size.
    /// - `seeds = [GAME_CONFIG_SEED]` + `bump` — derive the PDA and have
    ///   Anchor find the canonical bump (no off-curve retries needed on-chain).
    #[account(
        init,
        payer = admin,
        space = 8 + GameConfig::INIT_SPACE,
        seeds = [GAME_CONFIG_SEED],
        bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// The wallet paying for account creation and being recorded as admin.
    /// Must sign the transaction.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Required by the runtime to create accounts via CPI.
    pub system_program: Program<'info, System>,
}

/// Create the singleton `GameConfig` PDA and record the caller as admin.
///
/// Because [`InitializeGameConfig`]'s `init` constraint fails if the account
/// already exists, this instruction can only succeed once per deployment —
/// giving us a first-come-first-served admin model suitable for devnet. In a
/// production setting we would gate this further (e.g. a hardcoded admin key
/// baked into the program).
pub fn handler(ctx: Context<InitializeGameConfig>) -> Result<()> {
    let game_config = &mut ctx.accounts.game_config;
    game_config.admin = ctx.accounts.admin.key();
    game_config.bump = ctx.bumps.game_config;

    msg!("GameConfig initialised — admin: {}", game_config.admin);
    Ok(())
}
