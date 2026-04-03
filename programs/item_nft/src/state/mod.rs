use anchor_lang::prelude::*;

/// Item NFT metadata
#[account]
pub struct ItemNFTMetadata {
    pub item_type: u8,
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub uri: String,
    pub bump: u8,
}

impl ItemNFTMetadata {
    pub const SPACE: usize = 8 + 1 + 32 + 32 + (4 + 200) + 1; // 200 chars for URI
}

pub const ITEM_NAMES: &[&str] = &[
    "Козацька шабля",
    "Посох старійшини",
    "Броня характерника",
    "Бойовий браслет",
];

pub const ITEM_DESCRIPTIONS: &[&str] = &[
    "Cossack Sabre - A mighty weapon of the Kozaks",
    "Elder's Staff - A symbol of wisdom and power",
    "Cossack Armor - Protection of the brave",
    "Battle Bracelet - A warrior's pride",
];
