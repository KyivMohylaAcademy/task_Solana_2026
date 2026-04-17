use anchor_lang::prelude::*;

/// Стан гравця в системі пошуку ресурсів.
/// PDA seeds: [b"player", player_wallet]
#[account]
pub struct Player {
    /// Адреса гаманця гравця
    pub owner: Pubkey,
    /// Timestamp останнього успішного пошуку ресурсів (Unix секунди)
    pub last_search_timestamp: i64,
    /// Bump seed для цього PDA
    pub bump: u8,
}

impl Player {
    pub const SPACE: usize = 8  // discriminator
        + 32                    // owner
        + 8                     // last_search_timestamp
        + 1;                    // bump
}

/// PDA-авторитет програми search для підписання CPI до resource_manager.
/// PDA seeds: [b"search_authority"]
#[account]
pub struct SearchAuthority {
    /// Bump seed
    pub bump: u8,
}

impl SearchAuthority {
    pub const SPACE: usize = 8 + 1;
}
