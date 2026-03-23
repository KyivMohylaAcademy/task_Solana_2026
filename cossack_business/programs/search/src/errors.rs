use anchor_lang::prelude::*;

#[error_code]
pub enum SearchError {
    #[msg("Search cooldown has not expired yet")]
    SearchCooldown,
    #[msg("Timer overflow detected")]
    TimerOverflow,
    #[msg("Signer does not own this player account")]
    NotOwner,
    #[msg("Must pass exactly 12 remaining accounts (6 mints + 6 ATAs)")]
    InvalidRemainingAccounts,
}
