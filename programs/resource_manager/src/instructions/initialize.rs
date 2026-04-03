use anchor_lang::prelude::*;
use crate::state::GameConfig;

/// Initialize the game configuration and create resource mints
pub fn initialize_config(
    ctx: Context<InitializeConfig>,
    admin: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.game_config;
    config.admin = admin;
    config.bump = ctx.bumps.game_config;
    
    // Resource mints will be initialized separately via CPI from deployment script
    // For now, set to default
    config.resource_mints = [Pubkey::default(); 6];
    
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = payer,
        space = GameConfig::SPACE,
        seeds = [b"game_config"],
        bump
    )]
    pub game_config: Account<'info, GameConfig>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
