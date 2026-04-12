use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{burn, Burn, Mint, TokenAccount};

use crate::constants::*;
use crate::errors::ResourceManagerError;
use crate::state::GameConfig;

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct BurnResource<'info> {
    /// GameConfig PDA.
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// Мінт ресурсу, що спалюється.
    #[account(
        mut,
        constraint = game_config.resource_mints[resource_id as usize] == resource_mint.key()
            @ ResourceManagerError::InvalidResourceId,
    )]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    /// Токен-акаунт гравця — звідси списуються токени.
    #[account(
        mut,
        token::mint = resource_mint,
        token::authority = player,
        token::token_program = token_program,
    )]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Гравець — підписує транзакцію (власник токен-акаунту).
    pub player: Signer<'info>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<BurnResource>, resource_id: u8, amount: u64) -> Result<()> {
    require!(
        (resource_id as usize) < RESOURCE_COUNT as usize,
        ResourceManagerError::InvalidResourceId
    );

    burn(
        CpiContext::new(
            anchor_spl::token_2022::ID,
            Burn {
                mint: ctx.accounts.resource_mint.to_account_info(),
                from: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.player.to_account_info(),
            },
        ),
        amount,
    )?;

    msg!(
        "Спалено ресурс {}: {} одиниць від {}",
        resource_id,
        amount,
        ctx.accounts.player.key()
    );
    Ok(())
}
