use anchor_lang::prelude::*;

#[error_code]
pub enum ResourceManagerError {
    #[msg("Invalid resource index")]
    InvalidResourceIndex,
    #[msg("Insufficient resources")]
    InsufficientResources,
    #[msg("Unauthorized admin")]
    UnauthorizedAdmin,
}
