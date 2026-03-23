use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;
use resource_manager::program::ResourceManager;
use resource_manager::{self as rm};
use crate::state::Player;
use crate::errors::SearchError;

/// Accounts for the search instruction, which generates weighted-random
/// resources via CPI to resource_manager.
#[derive(Accounts)]
pub struct SearchResources<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [b"player", player.key().as_ref()],
        bump = player_account.bump,
        constraint = player_account.owner == player.key() @ SearchError::NotOwner,
    )]
    pub player_account: Account<'info, Player>,

    /// CHECK: PDA of this program used as CPI signer
    #[account(
        seeds = [b"caller_authority"],
        bump,
    )]
    pub caller_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub game_config: Account<'info, rm::GameConfig>,

    /// CHECK: resource_manager mint authority PDA
    #[account(
        seeds = [b"mint_authority"],
        bump = game_config.mint_authority_bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub mint_authority: UncheckedAccount<'info>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
