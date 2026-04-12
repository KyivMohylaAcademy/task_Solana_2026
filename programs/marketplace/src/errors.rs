use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceError {
    #[msg("NFT не виставлена на продаж.")]
    NotListed,
    #[msg("Тільки продавець може скасувати лістинг.")]
    NotSeller,
    #[msg("Ціна лістингу не може бути нулем.")]
    ZeroPrice,
}
