use anchor_lang::prelude::*;

declare_id!("5EyYkXzfHkH278x25q42csiR8FLeGvujpqCYdhncfcUd");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::{BuyItem, ListItem};
pub use instructions::*;

#[program]
pub mod marketplace {
    use super::*;

    /// List an item for sale
    pub fn list_item(
        ctx: Context<ListItem>,
        price: u64,
    ) -> Result<()> {
        instructions::list_item::list_item(ctx, price)
    }

    /// Buy an item from marketplace
    pub fn buy_item(
        ctx: Context<BuyItem>,
    ) -> Result<()> {
        instructions::buy_item::buy_item(ctx)
    }
}
