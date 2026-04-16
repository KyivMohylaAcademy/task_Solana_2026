//! `item_nft` — mints and manages in-game item NFTs (Metaplex Token Metadata).
//! Anyone can mint; only the registered `marketplace` program can burn.
//! The 1-of-1 guarantee is enforced by Metaplex's master edition.

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("CJi4wPcNAJmDyaJQ1ybYmF1hKP6Xtm1RT3JR9S2MbGiX");

#[program]
pub mod item_nft {
    use super::*;

    /// Create the singleton `ItemConfig` PDA. See
    /// [`instructions::initialize_item_config`].
    pub fn initialize_item_config(ctx: Context<InitializeItemConfig>) -> Result<()> {
        instructions::initialize_item_config::handler(ctx)
    }

    /// Admin-only: register the deployed `marketplace` program ID. Required
    /// before any `burn_item_nft` CPI can succeed. See
    /// [`instructions::set_marketplace_program`].
    pub fn set_marketplace_program(
        ctx: Context<SetMarketplaceProgram>,
        marketplace_program: Pubkey,
    ) -> Result<()> {
        instructions::set_marketplace_program::handler(ctx, marketplace_program)
    }

    /// Mint a 1-of-1 NFT to `recipient`. Open to any caller. See
    /// [`instructions::mint_item_nft`].
    pub fn mint_item_nft(
        ctx: Context<MintItemNft>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::mint_item_nft::handler(ctx, name, symbol, uri)
    }

    /// Burn an NFT. Callable only by the registered `marketplace` program via
    /// CPI. See [`instructions::burn_item_nft`].
    pub fn burn_item_nft(ctx: Context<BurnItemNft>) -> Result<()> {
        instructions::burn_item_nft::handler(ctx)
    }
}
