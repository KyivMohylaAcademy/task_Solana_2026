use anchor_lang::prelude::*;

/// Player state with search timer
#[account]
pub struct PlayerSearch {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub bump: u8,
}

impl PlayerSearch {
    pub const SPACE: usize = 8 + 32 + 8 + 1;
    pub const SEARCH_INTERVAL: i64 = 60; // 60 seconds
}
