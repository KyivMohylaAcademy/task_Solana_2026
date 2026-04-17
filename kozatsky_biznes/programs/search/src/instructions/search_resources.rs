use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
};
use resource_manager::{
    self,
    cpi::accounts::MintResource,
    program::ResourceManager,
    state::{GameConfig, MintAuthority},
};

use crate::state::*;
use crate::error::SearchError;
use crate::constants::SEARCH_COOLDOWN_SECONDS;

/// Виконує пошук ресурсів: генерує псевдовипадковий ресурс та мінтить 3 одиниці гравцю.
///
/// Алгоритм вибору ресурсу:
/// - Використовує clock.slot та clock.unix_timestamp як seed
/// - Вибирає один із 6 ресурсів псевдовипадково
/// - Мінтить 3 одиниці обраного ресурсу через CPI до resource_manager
///
/// Обмеження: між пошуками має пройти не менше 60 секунд.
#[derive(Accounts)]
pub struct SearchResources<'info> {
    /// Конфігурація гри (resource_manager)
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub game_config: Account<'info, GameConfig>,

    /// PDA-авторитет мінту resource_manager
    #[account(
        seeds = [b"mint_authority"],
        bump = mint_authority.bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub mint_authority: Account<'info, MintAuthority>,

    /// SearchAuthority PDA (підписує CPI до resource_manager)
    #[account(
        seeds = [b"search_authority"],
        bump = search_authority.bump,
    )]
    pub search_authority: Account<'info, SearchAuthority>,

    /// Акаунт гравця з таймстемпом останнього пошуку
    /// seeds включає player_wallet — неявна перевірка власника
    #[account(
        mut,
        seeds = [b"player", player_wallet.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == player_wallet.key() @ SearchError::UnauthorizedAdmin,
    )]
    pub player: Account<'info, Player>,

    /// Гаманець гравця (платник та підписант)
    #[account(mut)]
    pub player_wallet: Signer<'info>,

    /// Мінт ресурсного токена (вибраного псевдовипадково)
    /// CHECK: Перевіряється в resource_manager через game_config.resource_mints
    #[account(mut)]
    pub resource_mint: AccountInfo<'info>,

    /// Асоційований токен-акаунт гравця для ресурсу
    #[account(
        init_if_needed,
        payer = player_wallet,
        associated_token::mint = resource_mint,
        associated_token::authority = player_wallet,
        associated_token::token_program = token_2022_program,
    )]
    pub player_token_account: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SearchResources>) -> Result<()> {
    // Перевіряємо таймаут (60 секунд між пошуками)
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    let last = ctx.accounts.player.last_search_timestamp;

    require!(
        now - last >= SEARCH_COOLDOWN_SECONDS,
        SearchError::CooldownNotElapsed
    );

    // Псевдовипадковий вибір ресурсу на основі clock
    let slot = clock.slot;
    let timestamp = clock.unix_timestamp as u64;
    let seed = slot
        .wrapping_add(timestamp)
        .wrapping_add(ctx.accounts.player.last_search_timestamp as u64);
    let resource_id = (seed % 6) as u8;

    // Перевіряємо, що переданий мінт відповідає вибраному ресурсу
    require!(
        ctx.accounts.resource_mint.key()
            == ctx.accounts.game_config.resource_mints[resource_id as usize],
        SearchError::InvalidResourceMint
    );

    let search_bump = ctx.accounts.search_authority.bump;
    let search_seeds: &[&[u8]] = &[b"search_authority", &[search_bump]];

    // CPI до resource_manager::mint_resource — мінтимо 3 одиниці обраного ресурсу
    resource_manager::cpi::mint_resource(
        CpiContext::new_with_signer(
            ctx.accounts.resource_manager_program.key(),
            MintResource {
                game_config: ctx.accounts.game_config.to_account_info(),
                mint_authority: ctx.accounts.mint_authority.to_account_info(),
                caller_authority: ctx.accounts.search_authority.to_account_info(),
                payer: ctx.accounts.player_wallet.to_account_info(),
                player: ctx.accounts.player_wallet.to_account_info(),
                mint: ctx.accounts.resource_mint.to_account_info(),
                player_token_account: ctx.accounts.player_token_account.to_account_info(),
                token_2022_program: ctx.accounts.token_2022_program.to_account_info(),
                associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
            &[search_seeds],
        ),
        resource_id,
        3,
    )?;

    // Оновлюємо timestamp останнього пошуку
    ctx.accounts.player.last_search_timestamp = now;

    msg!(
        "Гравець {} знайшов ресурс {} (3 одиниці)",
        ctx.accounts.player_wallet.key(),
        resource_id
    );
    Ok(())
}
