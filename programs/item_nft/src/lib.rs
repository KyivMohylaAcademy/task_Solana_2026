use anchor_lang::prelude::*;

declare_id!("EEvis5VXdC5NRn4BrfUYdwcpzgptrfJZ3PsZ2UVxU8WY");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

#[program]
pub mod item_nft {
    use super::*;

    /// Мінтить NFT предмет через Metaplex Token Metadata.
    /// Тільки через CPI від crafting.
    pub fn create_item_nft(
        ctx: Context<CreateItemNft>,
        item_type: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::create_item_nft::handler(ctx, item_type, name, symbol, uri)
    }

    /// Спалює NFT предмет. Тільки через CPI від marketplace (buy_item).
    pub fn burn_item_nft(ctx: Context<BurnItemNft>) -> Result<()> {
        instructions::burn_item_nft::handler(ctx)
    }
}
