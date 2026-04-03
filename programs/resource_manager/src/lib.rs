use anchor_lang::prelude::*;

declare_id!("2Y2tAWf4DGPhk9kTDHyyProMw4wrNJf6R6U61WL8D4Vv");

pub mod errors;
pub mod instructions;
pub mod state;

pub use instructions::*;

#[program]
pub mod resource_manager {
    use super::*;

    /// Initialize the game config with resource mints
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        admin: Pubkey,
    ) -> Result<()> {
        instructions::initialize::initialize_config(ctx, admin)
    }

    /// Mint resources (called from other programs via CPI)
    pub fn mint_resource(
        ctx: Context<MintResource>,
        resource_index: u8,
        amount: u64,
    ) -> Result<()> {
        instructions::mint::mint_resource(ctx, resource_index, amount)
    }

    /// Burn resources (called from other programs via CPI)
    pub fn burn_resource(
        ctx: Context<BurnResource>,
        resource_index: u8,
        amount: u64,
    ) -> Result<()> {
        instructions::burn::burn_resource(ctx, resource_index, amount)
    }
}
