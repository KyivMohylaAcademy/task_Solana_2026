use anchor_lang::prelude::*;

/// Configuration PDA for the item NFT program, storing authorized
/// crafting and marketplace program IDs and the NFT authority bump.
#[account]
#[derive(InitSpace)]
pub struct ItemNftConfig {
    pub admin: Pubkey,
    pub crafting_program: Pubkey,
    pub marketplace_program: Pubkey,
    pub bump: u8,
    pub nft_authority_bump: u8,
}

/// On-chain metadata for a crafted NFT item, tracking its type, owner, and mint.
#[account]
#[derive(InitSpace)]
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}
