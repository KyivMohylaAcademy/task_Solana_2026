use anchor_lang::prelude::*;

use crate::{constants::ITEM_CONFIG_SEED, state::ItemConfig};

#[derive(Accounts)]
pub struct SetMarketplaceProgram<'info> {
    #[account(
        mut,
        seeds = [ITEM_CONFIG_SEED],
        bump = item_config.bump,
        has_one = admin,
    )]
    pub item_config: Account<'info, ItemConfig>,

    pub admin: Signer<'info>,
}

/// Register the deployed `marketplace` program ID with `ItemConfig`.
/// Required before any `burn_item_nft` CPI can succeed.
pub fn handler(ctx: Context<SetMarketplaceProgram>, marketplace_program: Pubkey) -> Result<()> {
    ctx.accounts.item_config.marketplace_program = marketplace_program;
    msg!("Registered marketplace program: {}", marketplace_program);
    Ok(())
}
