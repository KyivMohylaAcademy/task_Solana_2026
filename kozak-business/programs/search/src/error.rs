use anchor_lang::prelude::*;

#[error_code]
pub enum SearchError {
    #[msg("Search cooldown has not yet elapsed")]
    CooldownNotElapsed,
    #[msg("Mint account does not match the canonical resource_manager PDA for the given resource id")]
    InvalidResourceMint,
}
