use anchor_lang::prelude::*;

#[error_code]
pub enum ItemNftError {
    #[msg("Невірний тип предмета. Допустимо: 0–3.")]
    InvalidItemType,
    #[msg("Спалення NFT дозволено тільки через Marketplace (buy_item).")]
    UnauthorizedBurn,
    #[msg("NFT не належить гравцю.")]
    NotOwner,
}
