use anchor_lang::prelude::*;

#[error_code]
pub enum CraftingError {
    #[msg("Insufficient resources")]
    InsufficientResources,
    #[msg("Invalid item type")]
    InvalidItemType,
    #[msg("Wrong resource mint")]
    WrongResourceMint,
}
