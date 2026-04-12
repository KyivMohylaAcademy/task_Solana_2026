use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use resource_manager::cpi::accounts::MintResource;
use resource_manager::cpi::mint_resource;
use resource_manager::program::ResourceManager;
use resource_manager::state::GameConfig;

use crate::errors::SearchError;
use crate::state::Player;

use super::initialize_player::PLAYER_SEED;

pub const GAME_CONFIG_SEED: &[u8] = b"game_config";

/// remaining_accounts: [mint_0, ta_0, mint_1, ta_1, ..., mint_5, ta_5] — всі 6 пар.
/// Програма обирає 3 з них на основі псевдо-рандому.
#[derive(Accounts)]
pub struct SearchResources<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Player PDA гравця.
    #[account(
        mut,
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == owner.key(),
    )]
    pub player: Account<'info, Player>,

    /// GameConfig PDA з resource_manager.
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub game_config: Account<'info, GameConfig>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub token_program: Program<'info, Token2022>,
}

pub fn handler<'info>(ctx: Context<'info, SearchResources<'info>>) -> Result<()> {
    let clock = Clock::get()?;

    // Перевірка cooldown
    require!(
        clock.unix_timestamp - ctx.accounts.player.last_search_timestamp >= Player::SEARCH_COOLDOWN,
        SearchError::CooldownNotExpired
    );

    // Псевдо-рандом: LCG на базі timestamp + slot + owner
    let owner_bytes = ctx.accounts.player.owner.to_bytes();
    let seed: u64 = (clock.unix_timestamp as u64)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(clock.slot)
        ^ u64::from_le_bytes(owner_bytes[..8].try_into().unwrap());

    let resource_ids: [u8; 3] = [
        (seed % 6) as u8,
        ((seed.wrapping_mul(2862933555777941757).wrapping_add(3037000493)) % 6) as u8,
        ((seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) % 6) as u8,
    ];

    // Витягуємо інфо до циклу (уникаємо лайфтайм конфліктів)
    let game_config_info = ctx.accounts.game_config.to_account_info();
    let resource_mints = ctx.accounts.game_config.resource_mints;
    let owner_info = ctx.accounts.owner.to_account_info();
    let token_prog_info = ctx.accounts.token_program.to_account_info();

    // remaining_accounts: [mint_0, ta_0, mint_1, ta_1, ..., mint_5, ta_5]
    let remaining: Vec<AccountInfo> = ctx.remaining_accounts.iter().cloned().collect();
    require!(remaining.len() >= 12, SearchError::CooldownNotExpired); // reuse error for simplicity

    for resource_id in resource_ids.iter() {
        let idx = *resource_id as usize;
        let mint_info = remaining[idx * 2].clone();
        let ta_info = remaining[idx * 2 + 1].clone();

        // Перевіряємо відповідність мінту
        require!(
            resource_mints[idx] == mint_info.key(),
            SearchError::CooldownNotExpired
        );

        mint_resource(
            CpiContext::new(
                resource_manager::ID,
                MintResource {
                    game_config: game_config_info.clone(),
                    resource_mint: mint_info,
                    player_token_account: ta_info,
                    player: owner_info.clone(),
                    token_program: token_prog_info.clone(),
                },
            ),
            *resource_id,
            1,
        )?;

        msg!("Знайдено ресурс ID: {}", resource_id);
    }

    ctx.accounts.player.last_search_timestamp = clock.unix_timestamp;
    Ok(())
}
