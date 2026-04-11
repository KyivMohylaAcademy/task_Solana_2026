use anchor_lang::prelude::*;

declare_id!("5sk7gq8TwXpGFe7bxCsgWJ2k7StymKfXzkUD7HUfcMaY");

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

#[program]
pub mod magic_token {
    use super::*;

    /// Initialize the single Token-2022 MagicToken mint with embedded metadata.
    pub fn init_magic_token_mint(
        ctx: Context<InitMagicTokenMint>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::init_magic_token_mint::handler(ctx, name, symbol, uri)
    }

    /// Mint MagicToken to a recipient. Callable only by the marketplace program via CPI.
    pub fn mint_magic_token(
        ctx: Context<MintMagicToken>,
        amount: u64,
    ) -> Result<()> {
        instructions::mint_magic_token::handler(ctx, amount)
    }
}
