use anchor_lang::prelude::*;

#[error_code]
pub enum ResourceManagerError {
    #[msg("bad resource id")]
    InvalidResourceId,
    #[msg("mint/resource id mismatch")]
    InvalidMint,
    #[msg("unauthorized")]
    Unauthorized,
}
