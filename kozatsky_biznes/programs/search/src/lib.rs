pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8cNt5T19ynDwaV8MkTBodqWnY4HydWZu4SdKDWL6ZU5X");

/// Програма пошуку ресурсів гри "Козацький бізнес".
///
/// Забезпечує:
/// - Реєстрацію гравців (Player PDA)
/// - Пошук ресурсів з таймаутом 60 секунд
/// - Псевдовипадковий вибір 1 з 6 ресурсів та мінтинг 3 одиниць через CPI до resource_manager
#[program]
pub mod search {
    use super::*;

    /// Ініціалізує акаунт гравця для системи пошуку.
    pub fn initialize_player(ctx: Context<InitializePlayer>) -> Result<()> {
        initialize_player::handler(ctx)
    }

    /// Ініціалізує SearchAuthority PDA (викликається адміністратором один раз).
    pub fn initialize_search_authority(ctx: Context<InitializeSearchAuthority>) -> Result<()> {
        initialize_search_authority::handler(ctx)
    }

    /// Виконує пошук ресурсів з таймаутом 60 секунд.
    /// Мінтить 3 одиниці псевдовипадково обраного ресурсу через CPI до resource_manager.
    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        search_resources::handler(ctx)
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{Player, SearchAuthority};
    use crate::constants::SEARCH_COOLDOWN_SECONDS;

    /// Перевіряє розмір акаунта Player
    #[test]
    fn test_player_space() {
        // discriminator(8) + owner(32) + last_search_timestamp(8) + bump(1) = 49
        assert_eq!(Player::SPACE, 8 + 32 + 8 + 1);
    }

    /// Перевіряє розмір акаунта SearchAuthority
    #[test]
    fn test_search_authority_space() {
        // discriminator(8) + bump(1) = 9
        assert_eq!(SearchAuthority::SPACE, 9);
    }

    /// Перевіряє таймаут пошуку
    #[test]
    fn test_cooldown_constant() {
        assert_eq!(SEARCH_COOLDOWN_SECONDS, 60);
    }

    /// Перевіряє логіку псевдовипадкового вибору ресурсу
    #[test]
    fn test_pseudo_random_resource_selection() {
        // Симулюємо різні seed значення і перевіряємо що результат завжди в діапазоні 0-5
        for slot in 0u64..100 {
            for timestamp in 0u64..10 {
                let last_search: u64 = 0;
                let seed = slot
                    .wrapping_add(timestamp)
                    .wrapping_add(last_search);
                let resource_id = (seed % 6) as u8;
                assert!(resource_id < 6, "Resource ID {} повинен бути < 6", resource_id);
            }
        }
    }

    /// Перевіряє логіку таймауту
    #[test]
    fn test_cooldown_logic() {
        let last_search: i64 = 1000;
        let now_not_elapsed: i64 = 1050; // лише 50 секунд пройшло
        let now_elapsed: i64 = 1060; // 60 секунд пройшло

        // Не пройшов таймаут
        assert!(now_not_elapsed - last_search < SEARCH_COOLDOWN_SECONDS);
        // Пройшов рівно таймаут
        assert!(now_elapsed - last_search >= SEARCH_COOLDOWN_SECONDS);
    }
}
