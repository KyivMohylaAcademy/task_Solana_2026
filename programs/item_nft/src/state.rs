use anchor_lang::prelude::*;

/// Метадані NFT предмета. PDA seeds: ["item_metadata", mint].
#[account]
pub struct ItemMetadata {
    /// Тип предмета: 0=Шабля, 1=Посох, 2=Броня, 3=Браслет.
    pub item_type: u8,
    /// Власник NFT.
    pub owner: Pubkey,
    /// Адреса мінту NFT.
    pub mint: Pubkey,
    /// PDA bump.
    pub bump: u8,
}

impl ItemMetadata {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 1;
    pub const ITEM_TYPE_COUNT: u8 = 4;
}

pub const ITEM_NAMES: [&str; 4] = [
    "Шабля козака",
    "Посох старійшини",
    "Броня характерника",
    "Бойовий браслет",
];

pub const ITEM_SYMBOLS: [&str; 4] = ["SABER", "STAFF", "ARMOR", "BRACER"];
