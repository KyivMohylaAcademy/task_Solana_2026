use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{self, MintTo, Token2022},
};

use crate::state::*;
use crate::error::ResourceManagerError;

/// Мінтить ресурсні токени на токен-акаунт гравця.
///
/// Безпека: викликається виключно через CPI з програм search або crafting.
/// Верифікація: `caller_authority` має бути signer та відповідати
/// `game_config.search_authority` або `game_config.crafting_authority`.
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct MintResource<'info> {
    /// Конфігурація гри (зберігає авторизовані authority)
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// PDA-авторитет мінту (підписує MintTo CPI)
    #[account(
        seeds = [b"mint_authority"],
        bump = mint_authority.bump,
    )]
    pub mint_authority: Account<'info, MintAuthority>,

    /// Авторизований виклик (search або crafting authority PDA)
    /// CHECK: Перевіряється в коді: має бути signer і відповідати зареєстрованому authority
    #[account(
        mut,
        signer,
        constraint = (
            caller_authority.key() == game_config.search_authority
            || caller_authority.key() == game_config.crafting_authority
        ) @ ResourceManagerError::UnauthorizedCaller
    )]
    pub caller_authority: AccountInfo<'info>,

    /// Платник за відкриття токен-акаунта (зазвичай гравець або caller)
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Власник (гравець), що отримує ресурси
    /// CHECK: Просто адреса, токен-акаунт прив'язано до неї
    pub player: AccountInfo<'info>,

    /// Мінт ресурсного токена (Token-2022)
    /// CHECK: Перевіряється через game_config.resource_mints[resource_id]
    #[account(
        mut,
        constraint = mint.key() == game_config.resource_mints[resource_id as usize] @ ResourceManagerError::InvalidResourceId
    )]
    pub mint: AccountInfo<'info>,

    /// Асоційований токен-акаунт гравця (ініціалізується якщо не існує)
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = player,
        associated_token::token_program = token_2022_program,
    )]
    pub player_token_account: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<MintResource>, resource_id: u8, amount: u64) -> Result<()> {
    require!(resource_id < 6, ResourceManagerError::InvalidResourceId);

    let bump = ctx.accounts.mint_authority.bump;
    let seeds: &[&[u8]] = &[b"mint_authority", &[bump]];

    token_2022::mint_to(
        CpiContext::new_with_signer(
            anchor_spl::token_2022::ID,
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;

    msg!(
        "Мінт ресурсу {}: {} одиниць для гравця {}",
        resource_id,
        amount,
        ctx.accounts.player.key()
    );
    Ok(())
}
