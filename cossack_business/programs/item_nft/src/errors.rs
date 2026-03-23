use anchor_lang::prelude::*;

#[error_code]
pub enum ItemError {
    #[msg("Invalid item type (must be 0-3)")]
    InvalidItemType,
    #[msg("Signer is not the item owner")]
    NotOwner,
}
