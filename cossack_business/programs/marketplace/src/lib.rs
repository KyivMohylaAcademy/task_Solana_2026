use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("6mYp9XMhdaqcRq9xh4EDBmRDGaDEEphzEJzpPF5KEpvX");

#[program]
pub mod marketplace {
    use super::*;

    /// Sell an NFT to the game. Burns the NFT, mints MagicToken to seller.
    pub fn sell_item(ctx: Context<SellItem>) -> Result<()> {
        instructions::sell_item::handler(ctx)
    }
}
