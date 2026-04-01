//! Search program: 60-second cooldown and CPI to [`resource_manager`] for random resource mints.

use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use resource_manager::cpi::{accounts::MintSearchResources, mint_search_resources};
use shared::GameConfig;

declare_id!("FjtELP811XhkdKm63Hs2vc35fPdmGqf51YfE2cMMysQ2");

#[account]
pub struct Player {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub search_nonce: u64,
    pub bump: u8,
}

impl Player {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 1;
}

#[program]
pub mod search {
    use super::*;

    /// Initializes a [`Player`] PDA for `owner`.
    pub fn init_player(ctx: Context<InitPlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0;
        player.search_nonce = 0;
        player.bump = ctx.bumps.player;
        Ok(())
    }

    /// Runs a search: enforces cooldown, updates timestamp, then mints three random resource units.
    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let player = &mut ctx.accounts.player;
        require_keys_eq!(
            player.owner,
            ctx.accounts.owner.key(),
            SearchError::BadOwner
        );
        if player.last_search_timestamp != 0 && now - player.last_search_timestamp < 60 {
            return err!(SearchError::Cooldown);
        }
        player.last_search_timestamp = now;
        player.search_nonce = player.search_nonce.saturating_add(1);

        let owner_key = ctx.accounts.owner.key();
        let bump = ctx.bumps.search_authority;
        let seeds: &[&[u8]] = &[b"search", owner_key.as_ref(), &[bump]];
        let signer = &[seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.resource_manager_program.to_account_info(),
            MintSearchResources {
                game_config: ctx.accounts.game_config.to_account_info(),
                search_authority: ctx.accounts.search_authority.to_account_info(),
                player: ctx.accounts.owner.to_account_info(),
                resource_authority: ctx.accounts.resource_authority.to_account_info(),
                mint_wood: ctx.accounts.mint_wood.to_account_info(),
                mint_iron: ctx.accounts.mint_iron.to_account_info(),
                mint_gold: ctx.accounts.mint_gold.to_account_info(),
                mint_leather: ctx.accounts.mint_leather.to_account_info(),
                mint_stone: ctx.accounts.mint_stone.to_account_info(),
                mint_diamond: ctx.accounts.mint_diamond.to_account_info(),
                ata_wood: ctx.accounts.ata_wood.to_account_info(),
                ata_iron: ctx.accounts.ata_iron.to_account_info(),
                ata_gold: ctx.accounts.ata_gold.to_account_info(),
                ata_leather: ctx.accounts.ata_leather.to_account_info(),
                ata_stone: ctx.accounts.ata_stone.to_account_info(),
                ata_diamond: ctx.accounts.ata_diamond.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
            signer,
        );
        mint_search_resources(cpi_ctx)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitPlayer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub owner: Signer<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        space = Player::LEN,
        seeds = [b"player", owner.key().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SearchResources<'info> {
    pub owner: Signer<'info>,
    #[account(mut, seeds = [b"player", owner.key().as_ref()], bump = player.bump, has_one = owner)]
    pub player: Box<Account<'info, Player>>,
    #[account(
        mut,
        owner = resource_manager::ID,
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager::ID
    )]
    pub game_config: Box<Account<'info, GameConfig>>,
    /// CHECK: Search PDA for `owner`; constrained by seeds (invokes CPI with `invoke_signed`).
    #[account(seeds = [b"search", owner.key().as_ref()], bump)]
    pub search_authority: UncheckedAccount<'info>,
    /// CHECK: Resource mint authority PDA; constrained by seeds.
    #[account(
        seeds = [b"resource_auth", game_config.key().as_ref()],
        bump,
        seeds::program = resource_manager::ID
    )]
    pub resource_authority: UncheckedAccount<'info>,
    #[account(mut, address = game_config.resource_mints[0])]
    pub mint_wood: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[1])]
    pub mint_iron: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[2])]
    pub mint_gold: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[3])]
    pub mint_leather: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[4])]
    pub mint_stone: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[5])]
    pub mint_diamond: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub ata_wood: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_iron: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_gold: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_leather: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_stone: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_diamond: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Program<'info, Token2022>,
    /// CHECK: resource_manager program id
    #[account(address = resource_manager::ID)]
    pub resource_manager_program: UncheckedAccount<'info>,
}

#[error_code]
pub enum SearchError {
    #[msg("Search cooldown active")]
    Cooldown,
    #[msg("Player owner mismatch")]
    BadOwner,
}
