use anchor_lang::prelude::*;

declare_id!("DuCW5GarF2Z6Jb2SXTL42hViJ6SwdPvKJFJkq6wtP5eC");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

#[program]
pub mod marketplace {
    use super::*;

    /// Виставляє NFT предмет на продаж за вказану ціну в MagicToken.
    /// Створює Listing PDA, переводить NFT в ескроу.
    pub fn list_item(ctx: Context<ListItem>, price: u64) -> Result<()> {
        instructions::list_item::handler(ctx, price)
    }

    /// Купує предмет: спалює NFT через CPI → item_nft,
    /// мінтить MagicToken продавцю через CPI → magic_token.
    pub fn buy_item(ctx: Context<BuyItem>) -> Result<()> {
        instructions::buy_item::handler(ctx)
    }

    /// Скасовує лістинг і повертає NFT продавцю.
    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        instructions::cancel_listing::handler(ctx)
    }
}
