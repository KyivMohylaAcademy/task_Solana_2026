use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{mint_to, Mint, MintTo, TokenAccount};

use crate::constants::*;
use crate::errors::ResourceManagerError;
use crate::state::GameConfig;

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct MintResource<'info> {
    /// GameConfig PDA — виступає mint_authority.
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// Мінт конкретного ресурсу. Перевіряємо що він входить до game_config.resource_mints.
    #[account(
        mut,
        constraint = game_config.resource_mints[resource_id as usize] == resource_mint.key()
            @ ResourceManagerError::InvalidResourceId,
    )]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    /// Токен-акаунт гравця для цього ресурсу.
    #[account(
        mut,
        token::mint = resource_mint,
        token::authority = player,
        token::token_program = token_program,
    )]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Гравець — власник токен-акаунту.
    pub player: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<MintResource>, resource_id: u8, amount: u64) -> Result<()> {
    require!(
        (resource_id as usize) < RESOURCE_COUNT as usize,
        ResourceManagerError::InvalidResourceId
    );

    let seeds: &[&[u8]] = &[GAME_CONFIG_SEED, &[ctx.accounts.game_config.bump]];
    let signer_seeds = &[seeds];

    mint_to(
        CpiContext::new_with_signer(
            anchor_spl::token_2022::ID,
            MintTo {
                mint: ctx.accounts.resource_mint.to_account_info(),
                to: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.game_config.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    msg!(
        "Мінт ресурсу {}: {} одиниць для {}",
        resource_id,
        amount,
        ctx.accounts.player.key()
    );
    Ok(())
}
