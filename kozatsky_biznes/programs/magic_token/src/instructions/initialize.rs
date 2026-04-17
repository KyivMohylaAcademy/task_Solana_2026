use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token_2022::{self, InitializeMint2, Token2022};

use crate::state::*;
use crate::constants::BASIC_MINT_SIZE;

/// Ініціалізує конфігурацію MagicToken та створює Token-2022 мінт.
///
/// # Arguments
/// * `marketplace_authority` — адреса PDA програми marketplace (seeds=["marketplace_authority"])
///
/// Mint authority = PDA seeds=["mt_mint_authority"], decimals=0, необмежена емісія.
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Конфігурація MagicToken (PDA)
    #[account(
        init,
        payer = admin,
        space = MagicTokenConfig::SPACE,
        seeds = [b"magic_token_config"],
        bump,
    )]
    pub magic_token_config: Account<'info, MagicTokenConfig>,

    /// PDA-авторитет для підписання mint операцій
    #[account(
        init,
        payer = admin,
        space = MintAuthority::SPACE,
        seeds = [b"mt_mint_authority"],
        bump,
    )]
    pub mint_authority: Account<'info, MintAuthority>,

    /// Адміністратор (платник)
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Новий мінт-акаунт MagicToken (keypair від клієнта, signer)
    /// CHECK: Ініціалізується в цій інструкції через invoke до Token-2022
    #[account(mut, signer)]
    pub mint: AccountInfo<'info>,

    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<Initialize>, marketplace_authority: Pubkey) -> Result<()> {
    let mint_key = ctx.accounts.mint.key();
    let mint_auth_key = ctx.accounts.mint_authority.key();

    let lamports = ctx.accounts.rent.minimum_balance(BASIC_MINT_SIZE);

    // 1. Створюємо акаунт для мінту через SystemProgram
    invoke(
        &anchor_lang::solana_program::system_instruction::create_account(
            ctx.accounts.admin.key,
            &mint_key,
            lamports,
            BASIC_MINT_SIZE as u64,
            &anchor_spl::token_2022::ID,
        ),
        &[
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // 2. Ініціалізуємо мінт (decimals=0, mint_authority=PDA, freeze_authority=None)
    token_2022::initialize_mint2(
        CpiContext::new(
            anchor_spl::token_2022::ID,
            InitializeMint2 {
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        0,
        &mint_auth_key,
        None,
    )?;

    // 3. Зберігаємо конфігурацію
    let config = &mut ctx.accounts.magic_token_config;
    config.admin = ctx.accounts.admin.key();
    config.mint = mint_key;
    config.marketplace_authority = marketplace_authority;
    config.bump = ctx.bumps.magic_token_config;

    ctx.accounts.mint_authority.bump = ctx.bumps.mint_authority;

    msg!(
        "MagicToken ініціалізовано. Мінт: {}, Marketplace authority: {}",
        mint_key,
        marketplace_authority
    );
    Ok(())
}
