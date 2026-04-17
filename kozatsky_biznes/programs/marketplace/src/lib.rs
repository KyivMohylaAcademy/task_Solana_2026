pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("9tTHLWqsKEiVufU7uqeAgSJmJrvXTsCkSvTKgdrb2ofX");

/// Програма маркетплейсу гри "Козацький бізнес".
///
/// Забезпечує:
/// - Ініціалізацію MarketplaceAuthority PDA
/// - Продаж NFT предметів за MagicToken:
///   CPI до item_nft (спалення NFT) + CPI до magic_token (мінтинг нагороди)
#[program]
pub mod marketplace {
    use super::*;

    /// Ініціалізує MarketplaceAuthority PDA (викликається адміністратором один раз).
    pub fn initialize_marketplace_authority(
        ctx: Context<InitializeMarketplaceAuthority>,
    ) -> Result<()> {
        initialize_marketplace_authority::handler(ctx)
    }

    /// Продає NFT предмет гравця за MagicToken.
    pub fn sell_item(ctx: Context<SellItem>, item_type: u8) -> Result<()> {
        sell_item::handler(ctx, item_type)
    }
}

#[cfg(test)]
mod tests {
    use crate::state::MarketplaceAuthority;
    use crate::constants::ITEM_COUNT;

    /// Перевіряє розмір акаунта MarketplaceAuthority
    #[test]
    fn test_marketplace_authority_space() {
        // discriminator(8) + bump(1) = 9
        assert_eq!(MarketplaceAuthority::SPACE, 9);
    }

    /// Перевіряє кількість типів предметів
    #[test]
    fn test_item_count() {
        assert_eq!(ITEM_COUNT, 4);
    }

    /// Перевіряє логіку валідації типу предмету
    #[test]
    fn test_item_type_validation() {
        for item_type in 0u8..4 {
            assert!(
                (item_type as usize) < ITEM_COUNT,
                "item_type {} має бути < {}",
                item_type,
                ITEM_COUNT
            );
        }
        // item_type=4 має бути невалідним
        let invalid_type: u8 = 4;
        assert!(
            (invalid_type as usize) >= ITEM_COUNT,
            "item_type 4 має бути невалідним"
        );
    }
}
