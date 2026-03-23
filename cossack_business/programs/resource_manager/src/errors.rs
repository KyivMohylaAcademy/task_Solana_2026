use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    #[msg("Invalid resource ID (must be 0-5)")]
    InvalidResourceId,
    #[msg("Resources must be initialized in order")]
    OutOfOrder,
    #[msg("Invalid amount (must be > 0)")]
    InvalidAmount,
    #[msg("Resource mint does not match game config")]
    MintMismatch,
    #[msg("Unauthorized: signer is not admin")]
    Unauthorized,
    #[msg("Rarity weights must sum to 100")]
    InvalidRarityWeights,
    #[msg("Cooldown must be > 0")]
    InvalidCooldown,
    #[msg("Failed to calculate account space")]
    SpaceCalculationFailed,
}
