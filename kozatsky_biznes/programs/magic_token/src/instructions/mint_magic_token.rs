use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{self, MintTo, Token2022},
};

use crate::state::*;
use crate::error::MagicTokenError;

/// Мінтить MagicToken на токен-акаунт гравця.
///
/// Безпека: викликається виключно через CPI з програми marketplace.
/// Верифікація: `caller_authority` має бути signer та відповідати
/// `magic_token_config.marketplace_authority`.
#[derive(Accounts)]
pub struct MintMagicToken<'info> {
    /// Конфігурація MagicToken (зберігає авторизований marketplace authority)
    #[account(
        seeds = [b"magic_token_config"],
        bump = magic_token_config.bump,
    )]
    pub magic_token_config: Account<'info, MagicTokenConfig>,

    /// PDA-авторитет мінту (підписує MintTo CPI)
    #[account(
        seeds = [b"mt_mint_authority"],
        bump = mint_authority.bump,
    )]
    pub mint_authority: Account<'info, MintAuthority>,

    /// Авторизований виклик (marketplace authority PDA)
    /// CHECK: Перевіряється в коді: має бути signer та відповідати зареєстрованому authority
    #[account(
        signer,
        constraint = caller_authority.key() == magic_token_config.marketplace_authority
            @ MagicTokenError::UnauthorizedCaller
    )]
    pub caller_authority: AccountInfo<'info>,

    /// Платник за відкриття токен-акаунта (зазвичай гравець)
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Власник (гравець), що отримує MagicToken
    /// CHECK: Просто адреса, токен-акаунт прив'язано до неї
    pub player: AccountInfo<'info>,

    /// Мінт MagicToken
    /// CHECK: Перевіряється через magic_token_config.mint
    #[account(
        mut,
        constraint = mint.key() == magic_token_config.mint @ MagicTokenError::MintInitError
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

pub fn handler(ctx: Context<MintMagicToken>, amount: u64) -> Result<()> {
    let bump = ctx.accounts.mint_authority.bump;
    let seeds: &[&[u8]] = &[b"mt_mint_authority", &[bump]];

    // Мінтимо MagicToken на акаунт гравця, підписуючи PDA авторитетом
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
        "Мінт MagicToken: {} одиниць для гравця {}",
        amount,
        ctx.accounts.player.key()
    );
    Ok(())
}
