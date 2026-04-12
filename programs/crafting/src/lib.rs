use anchor_lang::prelude::*;

declare_id!("BqUaswbtzh21TNbcCpB2TnZEsVy6W3dAwgNZhDKEpc44");

pub mod constants;
pub mod errors;
pub mod instructions;

use instructions::*;

#[program]
pub mod crafting {
    use super::*;

    /// Крафтить предмет: перевіряє баланс ресурсів, спалює їх через CPI →
    /// resource_manager, мінтить NFT через CPI → item_nft.
    pub fn craft_item<'info>(
        ctx: Context<'info, CraftItem<'info>>,
        item_type: u8,
        metadata_uri: String,
    ) -> Result<()> {
        instructions::craft_item::handler(ctx, item_type, metadata_uri)
    }
}
