use anchor_lang::prelude::*;

/// Помилки програми MagicToken
#[error_code]
pub enum MagicTokenError {
    /// Виклик не авторизовано: тільки marketplace може мінтити MagicToken
    #[msg("Неавторизований виклик: тільки marketplace може мінтити MagicToken")]
    UnauthorizedCaller,
    /// Доступ заборонено: не адміністратор
    #[msg("Доступ заборонено: не адміністратор")]
    UnauthorizedAdmin,
    /// Мінт вже ініціалізовано
    #[msg("MagicToken мінт вже ініціалізовано")]
    AlreadyInitialized,
    /// Помилка ініціалізації Token-2022
    #[msg("Помилка ініціалізації Token-2022 мінту")]
    MintInitError,
}
