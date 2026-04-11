use anchor_lang::prelude::*;

/// Per-player PDA tracking search cooldown.
#[account]
pub struct Player {
    pub owner: Pubkey,
    /// Unix timestamp of the last successful search (0 = never searched).
    pub last_search_timestamp: i64,
    pub bump: u8,
}

impl Player {
    pub const LEN: usize = 8 + 32 + 8 + 1;
}
