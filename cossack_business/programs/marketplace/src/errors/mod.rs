use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceError {
    #[msg("seller has no nft")]
    NotNftHolder,
    #[msg("bad item type")]
    InvalidItemType,
}
