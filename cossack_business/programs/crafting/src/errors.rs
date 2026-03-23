use anchor_lang::prelude::*;

#[error_code]
pub enum CraftError {
    #[msg("Invalid item type (must be 0-3)")]
    InvalidItemType,
    #[msg("Wrong number of remaining accounts")]
    InvalidRemainingAccounts,
    #[msg("Missing resource for recipe requirement")]
    MissingResource,
    #[msg("Duplicate resource ID in resource_ids")]
    DuplicateResourceId,
}
