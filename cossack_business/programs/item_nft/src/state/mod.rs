use anchor_lang::prelude::*;

/// On-chain metadata for a minted NFT item.
#[account]
pub struct ItemMetadata {
    /// Item type index (0–3).
    pub item_type: u8,
    /// The wallet that originally received this NFT.
    pub owner: Pubkey,
    /// The NFT mint address.
    pub mint: Pubkey,
    /// PDA bump.
    pub bump: u8,
}

impl ItemMetadata {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 1;
}
