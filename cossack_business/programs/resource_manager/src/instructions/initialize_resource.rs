use anchor_lang::prelude::*;
use crate::state::GameConfig;
use crate::errors::GameError;

/// Accounts for creating a new Token-2022 resource mint with MetadataPointer.
#[derive(Accounts)]
#[instruction(id: u8)]
pub struct InitializeResource<'info> {
    #[account(
        mut,
        constraint = admin.key() == game_config.admin @ GameError::Unauthorized,
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// New mint keypair (must be a signer for create_account)
    #[account(mut)]
    pub mint: Signer<'info>,

    /// CHECK: PDA mint authority
    #[account(seeds = [b"mint_authority"], bump = game_config.mint_authority_bump)]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Token-2022 program
    #[account(address = spl_token_2022::ID)]
    pub token_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
