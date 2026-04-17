use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Burn, Token2022};

use crate::state::*;
use crate::error::ResourceManagerError;

/// Спалює ресурсні токени з токен-акаунту гравця.
///
/// Безпека: викликається виключно через CPI з програми crafting.
/// Верифікація: `caller_authority` має бути signer та відповідати
/// `game_config.crafting_authority`.
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct BurnResource<'info> {
    /// Конфігурація гри
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// Авторизований виклик (crafting authority PDA)
    /// CHECK: Має бути signer і відповідати game_config.crafting_authority
    #[account(
        signer,
        constraint = caller_authority.key() == game_config.crafting_authority
            @ ResourceManagerError::UnauthorizedBurnCaller
    )]
    pub caller_authority: AccountInfo<'info>,

    /// Гравець-власник токенів (підписує burn)
    #[account(mut)]
    pub player: Signer<'info>,

    /// Мінт ресурсного токена
    /// CHECK: Перевіряється через game_config.resource_mints[resource_id]
    #[account(
        mut,
        constraint = mint.key() == game_config.resource_mints[resource_id as usize]
            @ ResourceManagerError::InvalidResourceId
    )]
    pub mint: AccountInfo<'info>,

    /// Токен-акаунт гравця, з якого спалюємо
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = player,
        associated_token::token_program = token_2022_program,
    )]
    pub player_token_account: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    pub token_2022_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<BurnResource>, resource_id: u8, amount: u64) -> Result<()> {
    require!(resource_id < 6, ResourceManagerError::InvalidResourceId);

    token_2022::burn(
        CpiContext::new(
            anchor_spl::token_2022::ID,
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.player.to_account_info(),
            },
        ),
        amount,
    )?;

    msg!(
        "Спалено ресурс {}: {} одиниць у гравця {}",
        resource_id,
        amount,
        ctx.accounts.player.key()
    );
    Ok(())
}
