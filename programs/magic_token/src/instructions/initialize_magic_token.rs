use anchor_lang::prelude::*;
use anchor_spl::token_2022::{initialize_mint2, InitializeMint2, Token2022};
use anchor_spl::token_interface::Mint;

use crate::state::MagicConfig;

pub const MAGIC_CONFIG_SEED: &[u8] = b"magic_config";
pub const MAGIC_MINT_SEED: &[u8] = b"magic_mint";
pub const MAGIC_AUTHORITY_SEED: &[u8] = b"magic_authority";

#[derive(Accounts)]
pub struct InitializeMagicToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// MagicConfig PDA.
    #[account(
        init,
        payer = admin,
        space = MagicConfig::LEN,
        seeds = [MAGIC_CONFIG_SEED],
        bump,
    )]
    pub magic_config: Account<'info, MagicConfig>,

    /// MagicToken мінт — PDA з seeds ["magic_mint"].
    /// CHECK: Ініціалізується вручну через initialize_mint2 CPI.
    #[account(
        init,
        payer = admin,
        space = 82, // Mint account size для Token-2022 (без Anchor discriminator)
        seeds = [MAGIC_MINT_SEED],
        bump,
        owner = token_program.key(),
    )]
    pub magic_mint: UncheckedAccount<'info>,

    /// Marketplace PDA — стане mint_authority. Передається як UncheckedAccount.
    /// CHECK: Адреса Marketplace PDA перевіряється через seeds у Marketplace програмі.
    pub marketplace_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeMagicToken>) -> Result<()> {
    let magic_mint_bump = ctx.bumps.magic_mint;
    let magic_mint_seeds: &[&[u8]] = &[MAGIC_MINT_SEED, &[magic_mint_bump]];
    let signer_seeds = &[magic_mint_seeds];

    // Ініціалізуємо мінт з mint_authority = marketplace_authority
    initialize_mint2(
        CpiContext::new_with_signer(
            anchor_spl::token_2022::ID,
            InitializeMint2 {
                mint: ctx.accounts.magic_mint.to_account_info(),
            },
            signer_seeds,
        ),
        0, // decimals
        &ctx.accounts.marketplace_authority.key(),
        None, // freeze_authority
    )?;

    let config = &mut ctx.accounts.magic_config;
    config.mint = ctx.accounts.magic_mint.key();
    config.marketplace_authority = ctx.accounts.marketplace_authority.key();
    config.bump = ctx.bumps.magic_config;

    msg!("MagicToken ініціалізовано. Mint: {}", config.mint);
    Ok(())
}
