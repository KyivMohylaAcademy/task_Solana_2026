//! # search
//!
//! Allows players to search for resources once every `config.cooldown_seconds`.
//! On each search, 3 resources are pseudo-randomly selected and minted to the player
//! via CPI to `resource_manager`.
//!
//! ## Randomness
//! We read the first entry of the `SysvarRecentSlothashes` sysvar (8-byte slot + 32-byte hash),
//! XOR with `clock.slot`, `clock.unix_timestamp`, the player's pubkey, and a per-player nonce
//! to produce a 32-byte seed. This is acceptable pseudo-randomness for a game demo; it is not
//! MEV-resistant in production.

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint as InterfaceMint, TokenAccount as InterfaceTokenAccount},
};
use resource_manager::{
    self,
    cpi::{accounts::MintFromSearch as RmMintFromSearch, mint_from_search},
    program::ResourceManager,
};
use shared::{seeds::*, state::{GameConfig, Player}, errors::GameError};
use solana_program::hash::hashv;

declare_id!("9ZEk766xrSnSqJ4ke1vY9FhiGJwXZk37YK1ApBQaB6Pg");

/// Number of resources minted per search.
const RESOURCES_PER_SEARCH: usize = 3;
/// Total number of distinct resource kinds.
const RESOURCE_COUNT: u64 = 6;

#[program]
pub mod search {
    use super::*;

    /// Register a new player. Creates the Player PDA. Must be called before `run_search`.
    pub fn register_player(ctx: Context<RegisterPlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0;
        player.search_nonce = 0;
        player.bump = ctx.bumps.player;
        Ok(())
    }

    /// Run a resource search. Enforces the 60-second cooldown, generates 3 pseudo-random
    /// resources, and mints them to the player's ATAs via CPI to resource_manager.
    pub fn run_search(ctx: Context<RunSearch>) -> Result<()> {
        let clock = Clock::get()?;
        let config = &ctx.accounts.game_config;
        let player = &mut ctx.accounts.player;

        // ── Cooldown check ────────────────────────────────────────────────────
        require!(
            clock.unix_timestamp >= player.last_search_timestamp
                .checked_add(config.cooldown_seconds)
                .ok_or(GameError::Overflow)?,
            GameError::SearchTooSoon
        );

        // ── Pseudo-RNG seed ───────────────────────────────────────────────────
        // Read first entry of SysvarRecentSlothashes: [u64 slot | [u8;32] hash]
        let slothash_data = ctx.accounts.recent_slothashes.try_borrow_data()?;
        // Header is 8 bytes (entry count u64), then each entry is 8+32 = 40 bytes.
        // We read the first entry's hash (offset 8+8 = 16..48).
        require!(slothash_data.len() >= 48, GameError::Overflow);
        let slothash = &slothash_data[16..48];

        let seed = hashv(&[
            slothash,
            &clock.slot.to_le_bytes(),
            &clock.unix_timestamp.to_le_bytes(),
            player.owner.as_ref(),
            &player.search_nonce.to_le_bytes(),
        ]);
        let seed_bytes = seed.to_bytes();

        // ── Derive 3 resource kinds and amounts ───────────────────────────────
        let mut results: [(u8, u64); RESOURCES_PER_SEARCH] = [(0, 0); RESOURCES_PER_SEARCH];
        for i in 0..RESOURCES_PER_SEARCH {
            let kind_offset = i * 8;
            let kind_raw = u64::from_le_bytes(
                seed_bytes[kind_offset..kind_offset + 8].try_into().unwrap()
            );
            let kind = (kind_raw % RESOURCE_COUNT) as u8;
            // Amount: 1..=3 based on single byte from the second half of the seed
            let amount = 1u64 + (seed_bytes[24 + i] as u64 % 3);
            results[i] = (kind, amount);
        }

        // ── CPI: mint each resource ───────────────────────────────────────────
        let search_authority_bump = ctx.bumps.search_authority;
        let search_seeds: &[&[u8]] = &[SEARCH_AUTHORITY_SEED, &[search_authority_bump]];
        let signer_seeds = &[search_seeds];

        let rm_program = ctx.accounts.resource_manager_program.to_account_info();
        let rm_authority = ctx.accounts.resource_authority.to_account_info();

        // We pass 3 separate mints and ATAs; iterate over them using remaining_accounts.
        // Layout of remaining_accounts:
        //   [mint_0, ata_0, mint_1, ata_1, mint_2, ata_2]
        let remaining = ctx.remaining_accounts;
        require!(remaining.len() == RESOURCES_PER_SEARCH * 2, GameError::Overflow);

        for (i, (kind, amount)) in results.iter().enumerate() {
            let mint_info = &remaining[i * 2];
            let ata_info = &remaining[i * 2 + 1];

            mint_from_search(
                CpiContext::new_with_signer(
                    rm_program.clone(),
                    RmMintFromSearch {
                        search_authority: ctx.accounts.search_authority.to_account_info(),
                        resource_authority: rm_authority.clone(),
                        mint: mint_info.clone(),
                        player_ata: ata_info.clone(),
                        token_program: ctx.accounts.token_program.to_account_info(),
                    },
                    signer_seeds,
                ),
                *kind,
                *amount,
            )?;
        }

        // ── Update player state ───────────────────────────────────────────────
        player.last_search_timestamp = clock.unix_timestamp;
        player.search_nonce = player.search_nonce.checked_add(1).ok_or(GameError::Overflow)?;

        Ok(())
    }
}

// ─── Account structs ──────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct RegisterPlayer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = Player::LEN,
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump,
    )]
    pub player: Account<'info, Player>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RunSearch<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump = player.bump,
        has_one = owner,
    )]
    pub player: Account<'info, Player>,

    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// search_authority PDA – this program signs CPI calls with these seeds.
    /// CHECK: PDA, verified by seeds.
    #[account(
        seeds = [SEARCH_AUTHORITY_SEED],
        bump,
    )]
    pub search_authority: UncheckedAccount<'info>,

    /// resource_authority PDA from resource_manager (passed through to CPI).
    /// CHECK: Verified inside resource_manager.
    #[account(
        seeds = [RESOURCE_AUTHORITY_SEED],
        bump,
        seeds::program = resource_manager::ID,
    )]
    pub resource_authority: UncheckedAccount<'info>,

    /// CHECK: SysvarRecentSlothashes – read manually for RNG.
    #[account(address = solana_program::sysvar::recent_blockhashes::ID)]
    pub recent_slothashes: UncheckedAccount<'info>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    // remaining_accounts: [mint_0, ata_0, mint_1, ata_1, mint_2, ata_2]
}
