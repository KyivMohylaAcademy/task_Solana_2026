use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::TokenAccount;
use crate::state::MagicTokenConfig;
use crate::errors::MagicTokenError;

/// Mint MagicToken (only called from Marketplace via CPI)
pub fn mint_magic_token(
    ctx: Context<MintMagicToken>,
    amount: u64,
) -> Result<()> {
    let config = &ctx.accounts.config;
    
    // Only marketplace program can call this
    require_eq!(
        ctx.program_id,
        &ID,
        MagicTokenError::UnauthorizedMinter
    );
    
    // Mint tokens to recipient
    let cpi_accounts = token_2022::MintTo {
        mint: ctx.accounts.magic_mint.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_2022_program.to_account_info();
    token_2022::mint_to(CpiContext::new(cpi_program, cpi_accounts), amount)?;
    
    Ok(())
}

#[derive(Accounts)]
pub struct MintMagicToken<'info> {
    #[account(
        seeds = [b"magic_token_config"],
        bump = config.bump
    )]
    pub config: Account<'info, MagicTokenConfig>,
    
    pub magic_mint: Signer<'info>,
    
    #[account(mut)]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub authority: Signer<'info>,
    
    pub token_2022_program: Program<'info, Token2022>,
}

use crate::ID;
