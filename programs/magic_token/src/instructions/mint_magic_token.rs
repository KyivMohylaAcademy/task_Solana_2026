use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{mint_to, Mint, MintTo, TokenAccount};

use crate::state::MagicConfig;

use super::initialize_magic_token::MAGIC_CONFIG_SEED;

pub const MAGIC_AUTHORITY_SEED: &[u8] = b"magic_authority";

#[derive(Accounts)]
pub struct MintMagicToken<'info> {
    /// MagicConfig PDA.
    #[account(
        seeds = [MAGIC_CONFIG_SEED],
        bump = magic_config.bump,
    )]
    pub magic_config: Account<'info, MagicConfig>,

    /// MagicToken мінт.
    #[account(
        mut,
        constraint = magic_config.mint == magic_mint.key(),
    )]
    pub magic_mint: InterfaceAccount<'info, Mint>,

    /// Токен-акаунт отримувача (продавець на Marketplace).
    #[account(
        mut,
        token::mint = magic_mint,
        token::token_program = token_program,
    )]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Marketplace PDA — єдиний авторизований mint_authority.
    /// Має підписати транзакцію як CPI caller.
    #[account(
        constraint = magic_config.marketplace_authority == marketplace_authority.key(),
    )]
    pub marketplace_authority: Signer<'info>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<MintMagicToken>, amount: u64) -> Result<()> {
    // marketplace_authority підписує транзакцію — Signer constraint вже перевірено
    mint_to(
        CpiContext::new(
            anchor_spl::token_2022::ID,
            MintTo {
                mint: ctx.accounts.magic_mint.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.marketplace_authority.to_account_info(),
            },
        ),
        amount,
    )?;

    msg!(
        "MagicToken мінт: {} для {}",
        amount,
        ctx.accounts.recipient_token_account.key()
    );
    Ok(())
}
