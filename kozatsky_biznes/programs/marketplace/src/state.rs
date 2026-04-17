use anchor_lang::prelude::*;

/// PDA-авторитет програми marketplace для підписання CPI до item_nft та magic_token.
/// PDA seeds: [b"marketplace_authority"]
#[account]
pub struct MarketplaceAuthority {
    /// Bump seed
    pub bump: u8,
}

impl MarketplaceAuthority {
    pub const SPACE: usize = 8 + 1;
}
