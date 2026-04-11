use anchor_lang::prelude::*;

#[error_code]
pub enum ItemNftError {
    #[msg("bad item type")]
    InvalidItemType,
    #[msg("unauthorized")]
    Unauthorized,
}
