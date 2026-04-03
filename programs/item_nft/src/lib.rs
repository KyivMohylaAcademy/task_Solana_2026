use anchor_lang::prelude::*;

declare_id!("9GU3Nb13w1YaA8vwfLo2MqWmakbVLF9G6xZiNqCXn8ns");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::{BurnItemNFT, CreateItemNFT};
pub use instructions::*;

#[program]
pub mod item_nft {
    use super::*;

    /// Create a new item NFT
    pub fn create_item_nft(
        ctx: Context<CreateItemNFT>,
        item_type: u8,
        uri: String,
    ) -> Result<()> {
        instructions::create_item_nft::create_item_nft(ctx, item_type, uri)
    }

    /// Burn an item NFT
    pub fn burn_item_nft(
        ctx: Context<BurnItemNFT>,
    ) -> Result<()> {
        instructions::burn_item_nft::burn_item_nft(ctx)
    }
}
