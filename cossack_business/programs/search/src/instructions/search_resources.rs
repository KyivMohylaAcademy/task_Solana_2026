use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_2022::Token2022, token_interface::Mint};
use resource_manager::{
    self,
    cpi::accounts::MintResource,
    state::GameConfig,
};
use crate::{constants::SEARCH_COOLDOWN_SECONDS, errors::SearchError, state::Player};

#[derive(Accounts)]
pub struct SearchResources<'info> {
    /// Player wallet — must sign and must be the Player PDA owner.
    #[account(mut)]
    pub player_wallet: Signer<'info>,

    /// Player PDA. `has_one = owner` enforces player_wallet == player.owner.
    #[account(
        mut,
        seeds = [b"player", player_wallet.key().as_ref()],
        bump = player.bump,
        has_one = owner @ SearchError::Unauthorized,
    )]
    pub player: Account<'info, Player>,

    /// CHECK: The player wallet key, used for has_one validation.
    pub owner: AccountInfo<'info>,

    /// GameConfig PDA from resource_manager — provides the mint addresses.
    #[account(seeds = [b"game_config"], bump = game_config.bump, seeds::program = resource_manager::ID)]
    pub game_config: Account<'info, GameConfig>,

    /// CHECK: 6 resource mints (Token-2022). Each is pinned to game_config.resource_mints[i]
    /// and re-verified inside the mint_resource CPI.
    #[account(mut, constraint = mint0.key() == game_config.resource_mints[0])]
    pub mint0: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, constraint = mint1.key() == game_config.resource_mints[1])]
    pub mint1: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, constraint = mint2.key() == game_config.resource_mints[2])]
    pub mint2: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, constraint = mint3.key() == game_config.resource_mints[3])]
    pub mint3: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, constraint = mint4.key() == game_config.resource_mints[4])]
    pub mint4: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, constraint = mint5.key() == game_config.resource_mints[5])]
    pub mint5: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: 6 player ATAs — may be uninitialised; mint_resource CPI runs init_if_needed.
    #[account(mut)]
    pub ata0: UncheckedAccount<'info>,
    #[account(mut)]
    pub ata1: UncheckedAccount<'info>,
    #[account(mut)]
    pub ata2: UncheckedAccount<'info>,
    #[account(mut)]
    pub ata3: UncheckedAccount<'info>,
    #[account(mut)]
    pub ata4: UncheckedAccount<'info>,
    #[account(mut)]
    pub ata5: UncheckedAccount<'info>,

    /// CHECK: This program's cpi_auth PDA — passed to resource_manager as the caller identity.
    #[account(seeds = [b"cpi_auth"], bump)]
    pub cpi_auth: AccountInfo<'info>,

    /// CHECK: resource_mint_auth PDA from resource_manager — the actual mint authority.
    #[account(mut, seeds = [b"resource_mint_auth"], bump, seeds::program = resource_manager::ID)]
    pub resource_mint_auth: AccountInfo<'info>,

    /// CHECK: The resource_manager program. Verified by Anchor via the CPI call.
    #[account(address = resource_manager::ID)]
    pub resource_manager_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Search for resources. Mints 3 random resources to the player's ATAs.
/// Enforces SEARCH_COOLDOWN_SECONDS between calls.
pub fn handler(ctx: Context<SearchResources>) -> Result<()> {
    let clock = Clock::get()?;
    let player = &mut ctx.accounts.player;

    // Cooldown check.
    require!(
        clock.unix_timestamp - player.last_search_timestamp >= SEARCH_COOLDOWN_SECONDS,
        SearchError::CooldownNotElapsed
    );

    // Generate 3 resource IDs using slot + player key bytes for variation.
    let player_key_u64 =
        u64::from_le_bytes(player.key().to_bytes()[0..8].try_into().unwrap());

    let mints = [
        ctx.accounts.mint0.to_account_info(),
        ctx.accounts.mint1.to_account_info(),
        ctx.accounts.mint2.to_account_info(),
        ctx.accounts.mint3.to_account_info(),
        ctx.accounts.mint4.to_account_info(),
        ctx.accounts.mint5.to_account_info(),
    ];

    let atas = [
        ctx.accounts.ata0.to_account_info(),
        ctx.accounts.ata1.to_account_info(),
        ctx.accounts.ata2.to_account_info(),
        ctx.accounts.ata3.to_account_info(),
        ctx.accounts.ata4.to_account_info(),
        ctx.accounts.ata5.to_account_info(),
    ];

    let cpi_bump = ctx.bumps.cpi_auth;
    let signer_seeds: &[&[&[u8]]] = &[&[b"cpi_auth", &[cpi_bump]]];

    for i in 0u64..3 {
        let resource_id = ((clock.slot ^ player_key_u64 ^ i) % 6) as u8;

        let cpi_accounts = MintResource {
            cpi_auth: ctx.accounts.cpi_auth.to_account_info(),
            game_config: ctx.accounts.game_config.to_account_info(),
            mint: mints[resource_id as usize].clone(),
            recipient_ata: atas[resource_id as usize].clone(),
            recipient: ctx.accounts.player_wallet.to_account_info(),
            resource_mint_auth: ctx.accounts.resource_mint_auth.to_account_info(),
            payer: ctx.accounts.player_wallet.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };

        resource_manager::cpi::mint_resource(
            CpiContext::new_with_signer(
                ctx.accounts.resource_manager_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            ),
            resource_id,
            1,
        )?;
    }

    player.last_search_timestamp = clock.unix_timestamp;
    Ok(())
}
