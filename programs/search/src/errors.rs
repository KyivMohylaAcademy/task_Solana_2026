use anchor_lang::prelude::*;

#[error_code]
pub enum SearchError {
    #[msg("Cooldown не пройшов. Пошук доступний раз на 60 секунд.")]
    CooldownNotExpired,
    #[msg("Гравець вже ініціалізований.")]
    PlayerAlreadyInitialized,
}
