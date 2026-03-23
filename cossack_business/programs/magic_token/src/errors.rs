use anchor_lang::prelude::*;

#[error_code]
pub enum MagicError {
    #[msg("Invalid amount (must be > 0)")]
    InvalidAmount,
    #[msg("Mint does not match config")]
    MintMismatch,
    #[msg("Failed to calculate account space")]
    SpaceCalculationFailed,
}
