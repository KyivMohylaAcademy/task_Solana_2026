use anchor_lang::prelude::*;

/// Player account storing the wallet owner and last search timestamp
/// for enforcing the configurable cooldown between searches.
#[account]
#[derive(InitSpace)]
pub struct Player {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub bump: u8,
}
