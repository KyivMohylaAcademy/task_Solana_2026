use anchor_lang::prelude::*;

/// Глобальна конфігурація програми ItemNft.
/// PDA seeds: [b"item_nft_config"]
#[account]
pub struct ItemNftConfig {
    /// Адміністратор програми
    pub admin: Pubkey,
    /// Авторизована PDA програми crafting для виклику mint_nft_item
    pub crafting_authority: Pubkey,
    /// Авторизована PDA програми marketplace для виклику burn_nft_item
    pub marketplace_authority: Pubkey,
    /// Bump seed для цього PDA
    pub bump: u8,
}

impl ItemNftConfig {
    pub const SPACE: usize = 8  // discriminator
        + 32                    // admin
        + 32                    // crafting_authority
        + 32                    // marketplace_authority
        + 1;                    // bump
}

/// Метадані предмету, прив'язані до конкретного NFT мінту.
/// PDA seeds: [b"item_metadata", item_mint]
#[account]
pub struct ItemMetadata {
    /// Тип предмету: 0=Шабля козака, 1=Посох старійшини, 2=Броня характерника, 3=Бойовий браслет
    pub item_type: u8,
    /// Власник предмету (гравець)
    pub owner: Pubkey,
    /// Адреса NFT мінту
    pub mint: Pubkey,
    /// Bump seed для цього PDA
    pub bump: u8,
}

impl ItemMetadata {
    pub const SPACE: usize = 8  // discriminator
        + 1                     // item_type
        + 32                    // owner
        + 32                    // mint
        + 1;                    // bump
}

/// PDA-авторитет, що підписує операції мінтингу та спалення NFT.
/// PDA seeds: [b"nft_mint_authority"]
#[account]
pub struct NftMintAuthority {
    /// Bump seed
    pub bump: u8,
}

impl NftMintAuthority {
    pub const SPACE: usize = 8 + 1;
}
