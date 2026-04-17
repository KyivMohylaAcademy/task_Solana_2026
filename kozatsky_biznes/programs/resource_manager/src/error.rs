use anchor_lang::prelude::*;

#[error_code]
pub enum ResourceManagerError {
    #[msg("Доступ заборонено: не адміністратор")]
    UnauthorizedAdmin,
    #[msg("Недійсний ідентифікатор ресурсу (0-5)")]
    InvalidResourceId,
    #[msg("Неавторизований виклик: тільки search або crafting можуть мінтити ресурси")]
    UnauthorizedCaller,
    #[msg("Неавторизований виклик: тільки crafting може спалювати ресурси")]
    UnauthorizedBurnCaller,
    #[msg("Помилка ініціалізації розширення Token-2022")]
    ExtensionError,
    #[msg("Мінт для цього ресурсу вже встановлено")]
    MintAlreadySet,
}