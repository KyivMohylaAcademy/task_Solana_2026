use anchor_lang::prelude::*;

/// Per-wallet player account. Stored as a PDA derived from
/// [`crate::constants::PLAYER_SEED`] + the wallet's pubkey, so each wallet
/// has exactly one Player and the address is deterministic from the wallet.
#[account]
#[derive(InitSpace)]
pub struct Player {
    /// The wallet that owns this player record. Equal to the seed input,
    /// stored explicitly so off-chain code doesn't have to recompute it.
    pub wallet: Pubkey,
    /// Unix timestamp of the most recent successful search. `0` means the
    /// player has never searched — the cooldown check treats that as
    /// "always ready".
    pub last_search_timestamp: i64,
    /// Canonical bump for this PDA. Stored to avoid `find_program_address`
    /// on every subsequent instruction.
    pub bump: u8,
}
