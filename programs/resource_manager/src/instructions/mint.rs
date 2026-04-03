use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};
use crate::state::GameConfig;
use crate::errors::ResourceManagerError;

/// Mint resources (called from crafting/search programs via CPI)
pub fn mint_resource(
    ctx: Context<MintResource>,
    resource_index: u8,
    amount: u64,
) -> Result<()> {
    if resource_index >= 6 {
        return Err(ResourceManagerError::InvalidResourceIndex.into());
    }
    
    // Verify that the mint matches the configured mint for this resource
    require_eq!(
        ctx.accounts.resource_mint.key(),
        ctx.accounts.game_config.resource_mints[resource_index as usize],
        ResourceManagerError::InvalidResourceIndex
    );
    
    // Perform the mint via CPI
    let cpi_accounts = token_2022::MintTo {
        mint: ctx.accounts.resource_mint.to_account_info(),
        to: ctx.accounts.player_token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_2022_program.to_account_info();
    token_2022::mint_to(CpiContext::new(cpi_program, cpi_accounts), amount)?;
    
    Ok(())
}

#[derive(Accounts)]
pub struct MintResource<'info> {
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,
    
    pub resource_mint: InterfaceAccount<'info, Mint>,
    
    /// The player's token account to receive the resource
    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,
    
    /// PDA authority with mint permission
    pub authority: Signer<'info>,
    
    pub token_2022_program: Program<'info, Token2022>,
}
