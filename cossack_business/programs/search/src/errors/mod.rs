use anchor_lang::prelude::*;

#[error_code]
pub enum SearchError {
    #[msg("cooldown not elapsed")]
    CooldownNotElapsed,
    #[msg("unauthorized")]
    Unauthorized,
}
