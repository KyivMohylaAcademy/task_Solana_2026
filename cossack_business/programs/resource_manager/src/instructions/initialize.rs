use anchor_lang::prelude::*;
use crate::state::GameConfig;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = GameConfig::LEN,
        seeds = [b"game_config"],
        bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    pub system_program: Program<'info, System>,
}

/// Create the GameConfig PDA with all mint addresses and item prices.
pub fn handler(
    ctx: Context<Initialize>,
    resource_mints: [Pubkey; 6],
    magic_token_mint: Pubkey,
    item_prices: [u64; 4],
) -> Result<()> {
    let cfg = &mut ctx.accounts.game_config;
    cfg.admin = ctx.accounts.admin.key();
    cfg.resource_mints = resource_mints;
    cfg.magic_token_mint = magic_token_mint;
    cfg.item_prices = item_prices;
    cfg.bump = ctx.bumps.game_config;
    Ok(())
}
