use anchor_lang::prelude::*;

/// PDA-авторитет програми crafting для підписання CPI до resource_manager та item_nft.
/// PDA seeds: [b"crafting_authority"]
#[account]
pub struct CraftingAuthority {
    /// Bump seed
    pub bump: u8,
}

impl CraftingAuthority {
    pub const SPACE: usize = 8 + 1;
}
