use anchor_lang::prelude::*;

/// Помилки програми ItemNft
#[error_code]
pub enum ItemNftError {
    /// Виклик не авторизовано: тільки crafting може мінтити NFT предмети
    #[msg("Неавторизований виклик: тільки crafting може мінтити NFT предмети")]
    UnauthorizedMintCaller,
    /// Виклик не авторизовано: тільки marketplace може спалювати NFT предмети
    #[msg("Неавторизований виклик: тільки marketplace може спалювати NFT предмети")]
    UnauthorizedBurnCaller,
    /// Доступ заборонено: не адміністратор
    #[msg("Доступ заборонено: не адміністратор")]
    UnauthorizedAdmin,
    /// Недійсний тип предмету (0-3)
    #[msg("Недійсний тип предмету (0-3)")]
    InvalidItemType,
    /// Помилка ініціалізації Token-2022 розширення
    #[msg("Помилка ініціалізації Token-2022 розширення")]
    ExtensionError,
    /// NFT мінт не відповідає збереженому в метаданих
    #[msg("NFT мінт не відповідає збереженому в метаданих")]
    MintMismatch,
}
