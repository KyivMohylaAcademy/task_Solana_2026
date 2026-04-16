use anchor_lang::prelude::*;
use anchor_spl::token_2022::mint_to;
use anchor_spl::token_interface::{Mint, MintTo, TokenAccount, TokenInterface};

use crate::{
    constants::{GAME_CONFIG_SEED, MINT_AUTHORITY_SEED, RESOURCE_MINT_SEED, SEARCH_AUTHORITY_SEED},
    error::ResourceManagerError,
    state::GameConfig,
};

/// Accounts required by [`handler`]. The security spine of the whole system.
///
/// The crucial bit is `search_authority` being a `Signer` whose PDA derivation
/// is constrained to live under `game_config.search_program`. Anchor refuses
/// any caller that isn't the registered search program signing as that
/// canonical PDA.
#[derive(Accounts)]
#[instruction(resource_id: u8, amount: u64)]
pub struct MintResource<'info> {
    /// Source of truth for which program is allowed to drive this CPI.
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// PDA owned by the registered search program. The `seeds::program`
    /// constraint forces it to be derived under `game_config.search_program`,
    /// not the resource_manager program ID — so only the search program can
    /// produce the matching signature in a CPI.
    ///
    /// CHECK: Constraint enforced by Anchor via seeds + seeds::program.
    #[account(
        seeds = [SEARCH_AUTHORITY_SEED],
        bump,
        seeds::program = game_config.search_program,
    )]
    pub search_authority: Signer<'info>,

    /// The mint authority PDA for this specific resource. We sign as this PDA
    /// to satisfy Token-2022's mint_to authority check.
    ///
    /// CHECK: Constraint enforced by Anchor via seeds.
    #[account(
        seeds = [MINT_AUTHORITY_SEED, &[resource_id]],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// The resource's mint. Must be the canonical PDA for `resource_id`.
    #[account(
        mut,
        seeds = [RESOURCE_MINT_SEED, &[resource_id]],
        bump,
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// Destination token account. The `token::mint = mint` constraint ensures
    /// callers can't mint into an unrelated ATA.
    #[account(
        mut,
        token::mint = mint,
        token::token_program = token_program,
    )]
    pub recipient_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

/// Mint `amount` units of resource `resource_id` to `recipient_ata`.
///
/// Two layers of gating:
///   1. Outer: only the search program can satisfy `search_authority` (via
///      `seeds::program = game_config.search_program`).
///   2. Inner: only this program holds the seeds to sign as `mint_authority`,
///      so even on-chain code with this program's ID can't shortcut the mint.
pub fn handler(ctx: Context<MintResource>, resource_id: u8, amount: u64) -> Result<()> {
    require!(
        ctx.accounts.game_config.search_program != Pubkey::default(),
        ResourceManagerError::SearchProgramNotRegistered
    );

    require!(
        ctx.accounts.game_config.resource_mints[resource_id as usize] == ctx.accounts.mint.key(),
        ResourceManagerError::ResourceMintNotInitialised
    );

    let resource_id_bytes = [resource_id];
    let mint_authority_bump = ctx.bumps.mint_authority;
    let mint_authority_bump_bytes = [mint_authority_bump];
    let authority_seeds: &[&[u8]] = &[
        MINT_AUTHORITY_SEED,
        &resource_id_bytes,
        &mint_authority_bump_bytes,
    ];
    let signer_seeds: &[&[&[u8]]] = &[authority_seeds];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    Ok(())
}
