use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};
use crate::state::GameConfig;
use crate::errors::ResourceManagerError;

/// Burn resources (called from crafting program via CPI)
pub fn burn_resource(
    ctx: Context<BurnResource>,
    resource_index: u8,
    amount: u64,
) -> Result<()> {
    if resource_index >= 6 {
        return Err(ResourceManagerError::InvalidResourceIndex.into());
    }
    
    // Verify that the mint matches the configured mint
    require_eq!(
        ctx.accounts.resource_mint.key(),
        ctx.accounts.game_config.resource_mints[resource_index as usize],
        ResourceManagerError::InvalidResourceIndex
    );
    
    // Perform the burn via CPI
    let cpi_accounts = token_2022::Burn {
        mint: ctx.accounts.resource_mint.to_account_info(),
        from: ctx.accounts.player_token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_2022_program.to_account_info();
    token_2022::burn(CpiContext::new(cpi_program, cpi_accounts), amount)?;
    
    Ok(())
}

#[derive(Accounts)]
pub struct BurnResource<'info> {
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,
    
    pub resource_mint: InterfaceAccount<'info, Mint>,
    
    /// The player's token account from which to burn
    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,
    
    /// Player must sign to authorize the burn
    pub authority: Signer<'info>,
    
    pub token_2022_program: Program<'info, Token2022>,
}
