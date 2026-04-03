use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceError {
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Item not found")]
    ItemNotFound,
}
