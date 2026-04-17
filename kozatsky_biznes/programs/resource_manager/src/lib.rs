pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("G2FcuoLPQ8kSTTVLmKT9HR7b23em154skEidRVwTRtc9");

/// Програма для керування SPL Token-2022 ресурсними мінтами гри "Козацький бізнес".
///
/// Забезпечує:
/// - Ініціалізацію конфігурації гри та PDA-авторитету
/// - Створення 6 ресурсних мінтів (Token-2022 з MetadataPointer + TokenMetadata)
/// - Мінтинг ресурсів (тільки через search або crafting програми via CPI)
/// - Спалення ресурсів (тільки через crafting програму via CPI)
#[program]
pub mod resource_manager {
    use super::*;

    /// Ініціалізує конфігурацію гри та PDA-авторитет для мінтингу.
    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        search_authority: Pubkey,
        crafting_authority: Pubkey,
        item_prices: [u64; 4],
    ) -> Result<()> {
        initialize_game::handler(ctx, search_authority, crafting_authority, item_prices)
    }

    /// Створює SPL Token-2022 мінт з MetadataPointer та TokenMetadata для ресурсу.
    pub fn create_resource_mint(
        ctx: Context<CreateResourceMint>,
        resource_id: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        create_resource_mint::handler(ctx, resource_id, name, symbol, uri)
    }

    /// Мінтить ресурсні токени на акаунт гравця.
    /// Може викликатись тільки програмами search або crafting через CPI.
    pub fn mint_resource(
        ctx: Context<MintResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        mint_resource::handler(ctx, resource_id, amount)
    }

    /// Спалює ресурсні токени з акаунту гравця.
    /// Може викликатись тільки програмою crafting через CPI.
    pub fn burn_resource(
        ctx: Context<BurnResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        burn_resource::handler(ctx, resource_id, amount)
    }
}
