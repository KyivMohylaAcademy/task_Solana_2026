use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::state::GameConfig;
use crate::errors::GameError;

/// Accounts for CPI-gated resource minting (search program only).
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct MintResource<'info> {
    #[account(
        seeds = [b"caller_authority"],
        bump,
        seeds::program = game_config.search_program,
    )]
    pub caller_authority: Signer<'info>,

    #[account(seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,

    #[account(
        mut,
        constraint = resource_mint.key() == game_config.resource_mints[resource_id as usize]
            @ GameError::MintMismatch,
    )]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: PDA mint authority
    #[account(seeds = [b"mint_authority"], bump = game_config.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub player_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}
