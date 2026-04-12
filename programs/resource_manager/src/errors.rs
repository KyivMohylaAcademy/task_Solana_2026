use anchor_lang::prelude::*;

#[error_code]
pub enum ResourceManagerError {
    #[msg("Невірний ID ресурсу. Допустимо: 0–5.")]
    InvalidResourceId,
    #[msg("Недостатньо ресурсів на рахунку гравця.")]
    InsufficientResources,
    #[msg("Пряме мінтування заборонено. Використовуйте Search або Crafting.")]
    DirectMintingForbidden,
    #[msg("Неавторизований виклик. Тільки через CPI від дозволених програм.")]
    Unauthorized,
}
