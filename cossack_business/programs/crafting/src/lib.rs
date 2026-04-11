use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("YR3AszQR5gP98pMuzFb81Apb5KCsFi7U1gsSxfFeocF");

#[program]
pub mod crafting {
    use super::*;

    /// Craft an NFT item. Burns resources per recipe, mints NFT via item_nft CPI.
    pub fn craft_item(ctx: Context<CraftItem>, item_type: u8) -> Result<()> {
        instructions::craft_item::handler(ctx, item_type)
    }
}
