use anchor_lang::prelude::*;
use resource_manager::{self as rm, cpi::accounts::MintResource};

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("AFkvkn239iYeeNYkZ2gW7K4bUn1dNfwwQA8WuryTRBNr");

#[program]
pub mod search {
    use super::*;

    /// Register a new player account.
    pub fn register_player(ctx: Context<RegisterPlayer>) -> Result<()> {
        let acct = &mut ctx.accounts.player_account;
        acct.owner = ctx.accounts.player.key();
        acct.last_search_timestamp = 0;
        acct.bump = ctx.bumps.player_account;
        Ok(())
    }

    /// Search for resources. Enforces configurable cooldown and produces
    /// 3 weighted-random resources via CPI to resource_manager.
    pub fn search_resources<'info>(
        ctx: Context<'_, '_, 'info, 'info, SearchResources<'info>>,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let player_acct = &mut ctx.accounts.player_account;

        let cooldown = ctx.accounts.game_config.search_cooldown;
        let elapsed = clock
            .unix_timestamp
            .checked_sub(player_acct.last_search_timestamp)
            .ok_or(SearchError::TimerOverflow)?;
        require!(elapsed >= cooldown, SearchError::SearchCooldown);

        let seed_data = [
            clock.slot.to_le_bytes().as_ref(),
            clock.unix_timestamp.to_le_bytes().as_ref(),
            ctx.accounts.player.key().as_ref(),
        ]
        .concat();
        let hash = solana_program::hash::hashv(&[&seed_data]);
        let weights = ctx.accounts.game_config.rarity_weights;

        let caller_bump = ctx.bumps.caller_authority;
        let signer_seeds: &[&[u8]] = &[b"caller_authority", &[caller_bump]];

        let remaining = ctx.remaining_accounts;
        require!(
            remaining.len() == RESOURCE_COUNT * 2,
            SearchError::InvalidRemainingAccounts
        );

        for i in 0..RESOURCES_PER_SEARCH {
            let resource_id = pick_weighted_resource(hash.to_bytes()[i], &weights);
            let mint_idx = resource_id as usize;
            let ata_idx = RESOURCE_COUNT + resource_id as usize;

            rm::cpi::mint_resource(
                CpiContext::new_with_signer(
                    ctx.accounts.resource_manager_program.to_account_info(),
                    MintResource {
                        caller_authority: ctx.accounts.caller_authority.to_account_info(),
                        game_config: ctx.accounts.game_config.to_account_info(),
                        resource_mint: remaining[mint_idx].to_account_info(),
                        mint_authority: ctx.accounts.mint_authority.to_account_info(),
                        player_ata: remaining[ata_idx].to_account_info(),
                        token_program: ctx.accounts.token_program.to_account_info(),
                    },
                    &[signer_seeds],
                ),
                resource_id,
                1,
            )?;
        }

        player_acct.last_search_timestamp = clock.unix_timestamp;
        Ok(())
    }
}

/// Weighted random resource selection using admin-configured rarity weights.
fn pick_weighted_resource(rand_byte: u8, weights: &[u8; 6]) -> u8 {
    let roll = rand_byte % 100;
    let mut cumulative = 0u8;
    for (id, &weight) in weights.iter().enumerate() {
        cumulative = cumulative.saturating_add(weight);
        if roll < cumulative {
            return id as u8;
        }
    }
    0
}
