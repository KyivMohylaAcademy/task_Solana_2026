use anchor_lang::prelude::*;
use crate::state::*;

/// Ініціалізує глобальну конфігурацію гри та PDA-авторитет для мінтингу.
///
/// # Arguments
/// * `search_authority`   — адреса PDA програми search (seeds=["search_authority"])
/// * `crafting_authority` — адреса PDA програми crafting (seeds=["crafting_authority"])
/// * `item_prices`        — ціни предметів у MagicToken [шабля, посох, броня, браслет]
#[derive(Accounts)]
pub struct InitializeGame<'info> {
    /// Акаунт конфігурації гри (PDA)
    #[account(
        init,
        payer = admin,
        space = GameConfig::SPACE,
        seeds = [b"game_config"],
        bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// PDA-авторитет для підписання mint/burn операцій
    #[account(
        init,
        payer = admin,
        space = MintAuthority::SPACE,
        seeds = [b"mint_authority"],
        bump,
    )]
    pub mint_authority: Account<'info, MintAuthority>,

    /// Адміністратор гри (платник)
    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeGame>,
    search_authority: Pubkey,
    crafting_authority: Pubkey,
    item_prices: [u64; 4],
) -> Result<()> {
    let game_config = &mut ctx.accounts.game_config;
    let mint_authority = &mut ctx.accounts.mint_authority;

    game_config.admin = ctx.accounts.admin.key();
    game_config.resource_mints = [Pubkey::default(); 6];
    game_config.magic_token_mint = Pubkey::default();
    game_config.item_prices = item_prices;
    game_config.search_authority = search_authority;
    game_config.crafting_authority = crafting_authority;
    game_config.bump = ctx.bumps.game_config;

    mint_authority.bump = ctx.bumps.mint_authority;

    msg!("Гра ініціалізована. Адмін: {}", ctx.accounts.admin.key());
    Ok(())
}
