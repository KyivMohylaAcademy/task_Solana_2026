use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::state::MagicTokenConfig;
use crate::errors::MagicError;

/// Accounts for CPI-gated MagicToken minting (marketplace program only).
#[derive(Accounts)]
pub struct MintMagicToken<'info> {
    #[account(
        seeds = [b"caller_authority"],
        bump,
        seeds::program = config.marketplace_program,
    )]
    pub caller_authority: Signer<'info>,

    #[account(seeds = [b"magic_config"], bump = config.bump)]
    pub config: Account<'info, MagicTokenConfig>,

    #[account(
        mut,
        constraint = mint.key() == config.mint @ MagicError::MintMismatch,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: PDA mint authority
    #[account(seeds = [b"magic_mint_authority"], bump = config.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub recipient_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}
