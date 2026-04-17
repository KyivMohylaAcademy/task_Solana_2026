pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("7cSkPT8JwQHibWDnXCqsADcM5euk5RRSffRsz6U5QCkR");

/// Програма для керування NFT предметами гри "Козацький бізнес".
///
/// Забезпечує:
/// - Ініціалізацію конфігурації та PDA-авторитету
/// - Мінтинг NFT предметів (Token-2022, supply=1) з метаданими — тільки через crafting
/// - Спалення NFT предметів — тільки через marketplace
#[program]
pub mod item_nft {
    use super::*;

    /// Ініціалізує конфігурацію ItemNft та PDA-авторитет.
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        crafting_authority: Pubkey,
        marketplace_authority: Pubkey,
    ) -> Result<()> {
        initialize_config::handler(ctx, crafting_authority, marketplace_authority)
    }

    /// Мінтить NFT предмет на акаунт гравця.
    /// Може викликатись тільки програмою crafting через CPI.
    pub fn mint_nft_item(
        ctx: Context<MintNftItem>,
        item_type: u8,
        item_name: String,
        item_symbol: String,
        item_uri: String,
    ) -> Result<()> {
        mint_nft_item::handler(ctx, item_type, item_name, item_symbol, item_uri)
    }

    /// Спалює NFT предмет з акаунту гравця.
    /// Може викликатись тільки програмою marketplace через CPI.
    pub fn burn_nft_item(ctx: Context<BurnNftItem>) -> Result<()> {
        burn_nft_item::handler(ctx)
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{ItemNftConfig, ItemMetadata, NftMintAuthority};
    use crate::constants::{ITEM_NAMES, ITEM_SYMBOLS, ITEM_COUNT};

    /// Перевіряє розмір акаунта ItemNftConfig
    #[test]
    fn test_item_nft_config_space() {
        // discriminator(8) + admin(32) + crafting_authority(32) + marketplace_authority(32) + bump(1) = 105
        assert_eq!(ItemNftConfig::SPACE, 8 + 32 + 32 + 32 + 1);
    }

    /// Перевіряє розмір акаунта ItemMetadata
    #[test]
    fn test_item_metadata_space() {
        // discriminator(8) + item_type(1) + owner(32) + mint(32) + bump(1) = 74
        assert_eq!(ItemMetadata::SPACE, 8 + 1 + 32 + 32 + 1);
    }

    /// Перевіряє розмір акаунта NftMintAuthority
    #[test]
    fn test_nft_mint_authority_space() {
        // discriminator(8) + bump(1) = 9
        assert_eq!(NftMintAuthority::SPACE, 9);
    }

    /// Перевіряє кількість предметів та їх назви
    #[test]
    fn test_item_constants() {
        assert_eq!(ITEM_COUNT, 4);
        assert_eq!(ITEM_NAMES[0], "Шабля козака");
        assert_eq!(ITEM_NAMES[1], "Посох старійшини");
        assert_eq!(ITEM_NAMES[2], "Броня характерника");
        assert_eq!(ITEM_NAMES[3], "Бойовий браслет");
        assert_eq!(ITEM_SYMBOLS[0], "SABER");
        assert_eq!(ITEM_SYMBOLS[1], "STAFF");
        assert_eq!(ITEM_SYMBOLS[2], "ARMOR");
        assert_eq!(ITEM_SYMBOLS[3], "BRACELET");
    }
}
