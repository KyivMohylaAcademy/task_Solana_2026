use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("2DqgLTXd1joDVbtu3DSbocd8C9zExybcdzYH7a6gUXno");

#[program]
pub mod item_nft {
    use super::*;

    /// Mint a new NFT item of the given type to the recipient.
    pub fn mint_nft(ctx: Context<MintNft>, item_type: u8) -> Result<()> {
        instructions::mint_nft::handler(ctx, item_type)
    }

    /// Burn an NFT item. Callable only from the marketplace program via CPI.
    pub fn burn_nft(ctx: Context<BurnNft>) -> Result<()> {
        instructions::burn_nft::handler(ctx)
    }
}
