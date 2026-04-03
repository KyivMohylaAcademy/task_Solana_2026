use anchor_lang::prelude::*;
use crate::state::PlayerSearch;
use crate::errors::SearchError;

/// Search for resources - generates 3 random resources for the player
pub fn search_resources(
    ctx: Context<SearchResources>,
) -> Result<()> {
    let clock = Clock::get()?;
    let player_search = &mut ctx.accounts.player_search;

    if player_search.owner == Pubkey::default() {
        player_search.owner = ctx.accounts.owner.key();
        player_search.bump = ctx.bumps.player_search;
    }

    require_keys_eq!(
        player_search.owner,
        ctx.accounts.owner.key(),
        SearchError::UnauthorizedSearch
    );
    
    // Check if 60 seconds have passed since last search
    let time_elapsed = clock.unix_timestamp - player_search.last_search_timestamp;
    if time_elapsed < PlayerSearch::SEARCH_INTERVAL {
        return Err(SearchError::SearchNotReady.into());
    }
    
    // Update timestamp
    player_search.last_search_timestamp = clock.unix_timestamp;
    
    // Generate 3 random resources using clock slot as pseudo-randomness
    // In production, use a proper VRF (Verifiable Random Function) or oracle
    let slot = clock.slot;
    let seed1 = ((slot ^ 0x123) % 6) as u8;
    let seed2 = ((slot ^ 0x456) % 6) as u8;
    let seed3 = ((slot ^ 0x789) % 6) as u8;
    
    // Emit event with resource information
    emit!(ResourcesSearched {
        player: ctx.accounts.owner.key(),
        resources: [seed1, seed2, seed3],
        timestamp: clock.unix_timestamp,
    });
    
    // Actual minting would be done via CPI to resource_manager program
    // This instruction just validates the search attempt and updates timer
    
    Ok(())
}

#[derive(Accounts)]
pub struct SearchResources<'info> {
    #[account(
        init_if_needed,
        payer = owner,
        space = PlayerSearch::SPACE,
        seeds = [b"player_search", owner.key().as_ref()],
        bump
    )]
    pub player_search: Account<'info, PlayerSearch>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct ResourcesSearched {
    pub player: Pubkey,
    pub resources: [u8; 3],
    pub timestamp: i64,
}
