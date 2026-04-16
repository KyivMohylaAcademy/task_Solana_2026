use anchor_lang::prelude::*;

use crate::{constants::ITEM_CONFIG_SEED, state::ItemConfig};

#[derive(Accounts)]
pub struct InitializeItemConfig<'info> {
    /// The `ItemConfig` singleton PDA. `init` ensures it can only be created
    /// once; the caller becomes the registered admin.
    #[account(
        init,
        payer = admin,
        space = 8 + ItemConfig::INIT_SPACE,
        seeds = [ITEM_CONFIG_SEED],
        bump,
    )]
    pub item_config: Account<'info, ItemConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeItemConfig>) -> Result<()> {
    let item_config = &mut ctx.accounts.item_config;
    item_config.admin = ctx.accounts.admin.key();
    item_config.marketplace_program = Pubkey::default();
    item_config.bump = ctx.bumps.item_config;
    msg!("ItemConfig initialised — admin: {}", item_config.admin);
    Ok(())
}
