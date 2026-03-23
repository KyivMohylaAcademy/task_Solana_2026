use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::state::GameConfig;
use crate::errors::GameError;

/// Accounts for CPI-gated resource burning (crafting program only).
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct BurnResource<'info> {
    #[account(
        seeds = [b"caller_authority"],
        bump,
        seeds::program = game_config.crafting_program,
    )]
    pub caller_authority: Signer<'info>,

    pub player: Signer<'info>,

    #[account(seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,

    #[account(
        mut,
        constraint = resource_mint.key() == game_config.resource_mints[resource_id as usize]
            @ GameError::MintMismatch,
    )]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = resource_mint,
        token::authority = player,
    )]
    pub player_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}
