use anchor_lang::prelude::*;

#[error_code]
pub enum MarketError {
    #[msg("Invalid price (must be > 0)")]
    InvalidPrice,
    #[msg("Signer is not the item owner")]
    NotOwner,
    #[msg("Seller mismatch")]
    SellerMismatch,
    #[msg("Invalid item type")]
    InvalidItemType,
    #[msg("Mint does not match config")]
    MintMismatch,
    #[msg("Wrong number of remaining accounts")]
    InvalidRemainingAccounts,
}
