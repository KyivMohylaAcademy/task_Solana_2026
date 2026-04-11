use anchor_lang::prelude::*;

declare_id!("DFtQE4puDvEMk1vYHhx3gQvfjUieWj1YtkhDKoyGCG1y");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

#[program]
pub mod resource_manager {
    use super::*;

    /// Initialize a single Token-2022 resource mint with embedded metadata.
    pub fn init_resource_mint(
        ctx: Context<InitResourceMint>,
        resource_id: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::init_resource_mint::handler(ctx, resource_id, name, symbol, uri)
    }

    /// Create the GameConfig PDA with all mint addresses and item prices.
    pub fn initialize(
        ctx: Context<Initialize>,
        resource_mints: [Pubkey; 6],
        magic_token_mint: Pubkey,
        item_prices: [u64; 4],
    ) -> Result<()> {
        instructions::initialize::handler(ctx, resource_mints, magic_token_mint, item_prices)
    }

    /// Mint resource tokens. Callable only by the search or crafting program via CPI.
    pub fn mint_resource(
        ctx: Context<MintResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        instructions::mint_resource::handler(ctx, resource_id, amount)
    }

    /// Burn resource tokens. Callable only by the crafting program via CPI.
    pub fn burn_resource(
        ctx: Context<BurnResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        instructions::burn_resource::handler(ctx, resource_id, amount)
    }

    /// Admin-only resource minting for test setup. Gated by GameConfig.admin.
    pub fn admin_mint_resource(
        ctx: Context<AdminMintResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        instructions::admin_mint_resource::handler(ctx, resource_id, amount)
    }
}
