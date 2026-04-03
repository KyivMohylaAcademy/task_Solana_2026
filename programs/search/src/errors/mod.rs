use anchor_lang::prelude::*;

#[error_code]
pub enum SearchError {
    #[msg("Search timer not ready")]
    SearchNotReady,
    #[msg("Unauthorized search")]
    UnauthorizedSearch,
    #[msg("Invalid clock")]
    InvalidClock,
}
