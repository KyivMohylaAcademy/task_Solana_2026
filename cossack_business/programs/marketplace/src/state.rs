use anchor_lang::prelude::*;

/// Active marketplace listing with escrow, storing seller, NFT mint, price, and PDA bumps.
#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub seller: Pubkey,
    pub item_mint: Pubkey,
    pub item_type: u8,
    pub price: u64,
    pub bump: u8,
    pub escrow_bump: u8,
}
