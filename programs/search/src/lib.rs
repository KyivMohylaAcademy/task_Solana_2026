/// search — resource-discovery mechanic for "Козацький бізнес".
///
/// Each player has a [`PlayerAccount`] PDA that records the timestamp of their
/// last search.  A search can only be performed once every 60 seconds (on-chain
/// clock), preventing spam.
///
/// Pseudo-randomness
/// -----------------
/// True on-chain randomness requires a VRF oracle.  For this educational game,
/// we derive a pseudo-random seed from:
///   seed = sha256( player_pubkey || unix_timestamp || search_count )
/// This is deterministic but unpredictable enough for a game setting.
/// In production, use Switchboard VRF or a similar solution.
///
/// Cross-program invocations
/// -------------------------
/// `search_resources` calls `resource_manager::mint_resources` three times via
/// CPI, signing with the `search_authority` PDA of *this* program.
/// resource_manager verifies this authority against its `GameConfig`.
use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hashv;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};
use resource_manager::{
    self,
    cpi::{accounts::MintResources, mint_resources},
    program::ResourceManager,
};

declare_id!("Search111111111111111111111111111111111111111");

/// Cooldown between searches (seconds).
pub const SEARCH_COOLDOWN_SECS: i64 = 60;
/// Number of resources awarded per search.
pub const RESOURCES_PER_SEARCH: usize = 3;
/// Amount of each resource type awarded per find.
pub const RESOURCE_AMOUNT: u64 = 1;
/// Number of resource types in the game.
pub const NUM_RESOURCES: usize = 6;

// ─── Program ──────────────────────────────────────────────────────────────────

#[program]
pub mod search {
    use super::*;

    /// Initialises a [`PlayerAccount`] PDA for `player`.
    ///
    /// Must be called once per player before they can search.
    pub fn initialize_player(ctx: Context<InitializePlayer>) -> Result<()> {
        let acc = &mut ctx.accounts.player_account;
        acc.owner                = ctx.accounts.player.key();
        acc.last_search_timestamp = 0;  // allows first search immediately
        acc.search_count         = 0;
        acc.bump                 = ctx.bumps.player_account;
        Ok(())
    }

    /// Performs a resource search.
    ///
    /// Awards 3 pseudo-random resource tokens to the player.
    /// Fails if less than 60 seconds have elapsed since the last search.
    ///
    /// The caller must pass the 6 player token accounts in `remaining_accounts`
    /// (one per resource type, in order WOOD=0 … DIAMOND=5) and the
    /// corresponding resource mints, interleaved: mint_0, ta_0, mint_1, ta_1, …
    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        let clock = Clock::get()?;
        let now   = clock.unix_timestamp;

        let player_acc = &mut ctx.accounts.player_account;

        // ── Cooldown check ────────────────────────────────────────────────────
        require!(
            now.saturating_sub(player_acc.last_search_timestamp) >= SEARCH_COOLDOWN_SECS,
            SearchError::CooldownNotElapsed
        );

        // ── Derive 3 pseudo-random resource types ─────────────────────────────
        let player_key  = ctx.accounts.player.key();
        let count_bytes = player_acc.search_count.to_le_bytes();
        let ts_bytes    = now.to_le_bytes();
        let hash        = hashv(&[player_key.as_ref(), &ts_bytes, &count_bytes]);
        let hash_bytes  = hash.to_bytes();

        let mut resource_types = [0u8; RESOURCES_PER_SEARCH];
        for (i, rt) in resource_types.iter_mut().enumerate() {
            *rt = hash_bytes[i] % NUM_RESOURCES as u8;
        }

        // ── CPI: mint each resource via resource_manager ──────────────────────
        // remaining_accounts layout: [mint_0, ta_0, mint_1, ta_1, mint_2, ta_2]
        require!(
            ctx.remaining_accounts.len() == RESOURCES_PER_SEARCH * 2,
            SearchError::WrongAccountCount
        );

        let search_auth_bump = ctx.bumps.search_authority;
        let seeds: &[&[u8]] = &[b"search_authority", &[search_auth_bump]];
        let signer_seeds = &[seeds];

        for (i, &rt) in resource_types.iter().enumerate() {
            let mint_info = &ctx.remaining_accounts[i * 2];
            let ta_info   = &ctx.remaining_accounts[i * 2 + 1];

            mint_resources(
                CpiContext::new_with_signer(
                    ctx.accounts.resource_manager_program.to_account_info(),
                    MintResources {
                        game_config:          ctx.accounts.game_config.to_account_info(),
                        mint_authority:       ctx.accounts.resource_mint_authority.to_account_info(),
                        authority:            ctx.accounts.search_authority.to_account_info(),
                        resource_mint:        mint_info.clone(),
                        player_token_account: ta_info.clone(),
                        token_program:        ctx.accounts.token_program.to_account_info(),
                    },
                    signer_seeds,
                ),
                rt,
                RESOURCE_AMOUNT,
            )?;
        }

        // ── Update player state ───────────────────────────────────────────────
        player_acc.last_search_timestamp = now;
        player_acc.search_count          = player_acc.search_count.saturating_add(1);

        emit!(ResourcesFound {
            player:         ctx.accounts.player.key(),
            resource_types: resource_types,
            timestamp:      now,
        });
        Ok(())
    }
}

// ─── Accounts ─────────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct InitializePlayer<'info> {
    #[account(
        init,
        payer  = player,
        space  = PlayerAccount::LEN,
        seeds  = [b"player", player.key().as_ref()],
        bump,
    )]
    pub player_account: Account<'info, PlayerAccount>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SearchResources<'info> {
    #[account(
        mut,
        seeds = [b"player", player.key().as_ref()],
        bump  = player_account.bump,
    )]
    pub player_account: Account<'info, PlayerAccount>,

    pub player: Signer<'info>,

    /// This program's cross-program authority PDA.
    /// Recognised by resource_manager as an authorised minter.
    /// CHECK: PDA derived from seeds below
    #[account(seeds = [b"search_authority"], bump)]
    pub search_authority: UncheckedAccount<'info>,

    /// resource_manager's mint_authority PDA.
    /// CHECK: validated by resource_manager
    #[account(
        seeds = [b"mint_authority"],
        bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub resource_mint_authority: UncheckedAccount<'info>,

    /// resource_manager's GameConfig.
    /// CHECK: validated by resource_manager CPI
    pub game_config: UncheckedAccount<'info>,

    pub resource_manager_program: Program<'info, ResourceManager>,

    pub token_program: Program<'info, Token2022>,
}

// ─── State ────────────────────────────────────────────────────────────────────

/// Per-player account tracking search cooldown and statistics.
#[account]
pub struct PlayerAccount {
    /// The player's wallet pubkey.
    pub owner: Pubkey,
    /// Unix timestamp of the last successful search.
    pub last_search_timestamp: i64,
    /// Lifetime search count (used as PRNG nonce).
    pub search_count: u64,
    pub bump: u8,
}

impl PlayerAccount {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 1;
}

// ─── Events ───────────────────────────────────────────────────────────────────

#[event]
pub struct ResourcesFound {
    pub player:         Pubkey,
    pub resource_types: [u8; 3],
    pub timestamp:      i64,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[error_code]
pub enum SearchError {
    #[msg("Search cooldown has not elapsed — wait 60 seconds between searches")]
    CooldownNotElapsed,
    #[msg("Expected 6 (mint, token_account) pairs in remaining_accounts")]
    WrongAccountCount,
}
