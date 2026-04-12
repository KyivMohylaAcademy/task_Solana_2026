use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::state::GameConfig;

#[derive(Accounts)]
pub struct InitializeGameConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// GameConfig PDA — зберігає адреси всіх мінтів.
    #[account(
        init,
        payer = admin,
        space = GameConfig::LEN,
        seeds = [GAME_CONFIG_SEED],
        bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    // 6 ресурсних мінтів (SPL Token-2022), ініціалізуються окремо через init_resource_mint
    /// CHECK: Мінт ресурсу 0 (WOOD) — ініціалізується інструкцією init_resource_mint.
    pub mint_wood: UncheckedAccount<'info>,
    /// CHECK: Мінт ресурсу 1 (IRON).
    pub mint_iron: UncheckedAccount<'info>,
    /// CHECK: Мінт ресурсу 2 (GOLD).
    pub mint_gold: UncheckedAccount<'info>,
    /// CHECK: Мінт ресурсу 3 (LEATHER).
    pub mint_leather: UncheckedAccount<'info>,
    /// CHECK: Мінт ресурсу 4 (STONE).
    pub mint_stone: UncheckedAccount<'info>,
    /// CHECK: Мінт ресурсу 5 (DIAMOND).
    pub mint_diamond: UncheckedAccount<'info>,

    /// CHECK: MagicToken мінт — ініціалізується magic_token програмою.
    pub magic_token_mint: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<InitializeGameConfig>) -> Result<()> {
    let config = &mut ctx.accounts.game_config;
    config.admin = ctx.accounts.admin.key();
    config.resource_mints = [
        ctx.accounts.mint_wood.key(),
        ctx.accounts.mint_iron.key(),
        ctx.accounts.mint_gold.key(),
        ctx.accounts.mint_leather.key(),
        ctx.accounts.mint_stone.key(),
        ctx.accounts.mint_diamond.key(),
    ];
    config.magic_token_mint = ctx.accounts.magic_token_mint.key();
    config.item_prices = [10, 15, 20, 25]; // ціни у MagicToken
    config.bump = ctx.bumps.game_config;

    msg!("GameConfig ініціалізовано. Admin: {}", config.admin);
    Ok(())
}
