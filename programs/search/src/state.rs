use anchor_lang::prelude::*;

/// Player PDA — зберігає стан гравця. Seeds: ["player", owner].
#[account]
pub struct Player {
    /// Власник (гаманець гравця).
    pub owner: Pubkey,
    /// Unix timestamp останнього пошуку.
    pub last_search_timestamp: i64,
    /// PDA bump.
    pub bump: u8,
}

impl Player {
    pub const LEN: usize = 8 + 32 + 8 + 1;

    /// Cooldown між пошуками (секунди).
    pub const SEARCH_COOLDOWN: i64 = 60;

    /// Кількість ресурсів за один пошук.
    pub const RESOURCES_PER_SEARCH: u8 = 3;
}
