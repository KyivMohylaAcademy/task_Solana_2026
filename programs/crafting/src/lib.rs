use anchor_lang::prelude::*;

declare_id!("CTHKMpMxaV89e4g7a4uwmvPmSYygWvtFn4vv9qRQ5m2t");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::CraftItem;
pub use instructions::*;

#[program]
pub mod crafting {
    use super::*;

    /// Craft an item from resources
    pub fn craft_item(
        ctx: Context<CraftItem>,
        item_type: u8,
    ) -> Result<()> {
        instructions::craft_item::craft_item(ctx, item_type)
    }
}
