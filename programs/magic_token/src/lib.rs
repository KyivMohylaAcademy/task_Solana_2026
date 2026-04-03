use anchor_lang::prelude::*;

declare_id!("4NvPT6ob4cPTGpXDq9TEp5ByuW5HgxAYYCUWW5xDS6dE");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::MintMagicToken;
pub use instructions::*;

#[program]
pub mod magic_token {
    use super::*;

    /// Mint MagicToken (only from Marketplace via CPI)
    pub fn mint_magic_token(
        ctx: Context<MintMagicToken>,
        amount: u64,
    ) -> Result<()> {
        instructions::mint_magic::mint_magic_token(ctx, amount)
    }
}
