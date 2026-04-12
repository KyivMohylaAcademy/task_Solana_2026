use anchor_lang::prelude::*;

#[error_code]
pub enum MagicTokenError {
    #[msg("Пряме мінтування MagicToken заборонено. Тільки через Marketplace CPI.")]
    DirectMintingForbidden,
    #[msg("Неавторизований виклик. Mint authority — Marketplace PDA.")]
    Unauthorized,
}
