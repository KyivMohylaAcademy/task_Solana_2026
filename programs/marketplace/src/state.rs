use anchor_lang::prelude::*;

/// Listing PDA — зберігає інформацію про виставлений на продаж NFT.
/// Seeds: ["listing", nft_mint].
#[account]
pub struct Listing {
    /// Продавець (власник NFT).
    pub seller: Pubkey,
    /// Адреса мінту NFT.
    pub nft_mint: Pubkey,
    /// Ціна в MagicToken.
    pub price: u64,
    /// PDA bump.
    pub bump: u8,
}

impl Listing {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 1;
}
