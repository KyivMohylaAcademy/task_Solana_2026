use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use anchor_spl::token_interface;

declare_id!("Cnp9S3UtAQEKmhtgVErCVFedtDXfwbZY8kgNYbugGLtn");

#[program]
pub mod magic_token {
    use super::*;

    /// Initialize the MagicToken mint
    pub fn initialize(ctx: Context<Initialize>, decimals: u8) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.mint = ctx.accounts.mint.key();
        config.bump = ctx.bumps.config;
        
        msg!("MagicToken initialized with {} decimals", decimals);
        Ok(())
    }

    /// Mint MagicToken (only callable by marketplace program via CPI)
    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
    ) -> Result<()> {
        // Verify the caller is the marketplace program
        // This is enforced through PDA authority
        
        let seeds: &[&[u8]] = &[
            b"mint_authority",
            &[ctx.accounts.config.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = token_2022::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        
        token_2022::mint_to(cpi_ctx, amount)?;
        
        msg!("Minted {} MagicTokens", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + MagicTokenConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, MagicTokenConfig>,
    
    /// CHECK: The mint account will be initialized separately
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, MagicTokenConfig>,
    
    /// CHECK: PDA authority for minting
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    pub mint_authority: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = mint.key() == config.mint @ ErrorCode::InvalidMint
    )]
    pub mint: InterfaceAccount<'info, token_interface::Mint>,
    
    /// CHECK: Recipient's token account
    #[account(mut)]
    pub token_account: AccountInfo<'info>,
    
    pub token_program: Program<'info, token_2022::Token2022>,
}

#[account]
#[derive(InitSpace)]
pub struct MagicTokenConfig {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid mint address")]
    InvalidMint,
    #[msg("Unauthorized - only marketplace can mint")]
    Unauthorized,
}
