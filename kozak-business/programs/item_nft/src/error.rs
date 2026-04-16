use anchor_lang::prelude::*;

#[error_code]
pub enum ItemNftError {
    #[msg("Marketplace program has not been registered on the ItemConfig")]
    MarketplaceProgramNotRegistered,
}
