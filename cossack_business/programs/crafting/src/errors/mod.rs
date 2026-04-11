use anchor_lang::prelude::*;

#[error_code]
pub enum CraftingError {
    #[msg("bad item type")]
    InvalidItemType,
}
