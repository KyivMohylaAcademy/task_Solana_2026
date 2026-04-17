pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("9gbQXMjCev1K5GmK4SzBa52zKtXPveZQvVmaA15rvszs");

/// Програма крафтингу предметів гри "Козацький бізнес".
///
/// Забезпечує:
/// - Ініціалізацію CraftingAuthority PDA
/// - Крафт предметів: спалення ресурсів (CPI до resource_manager) та мінтинг NFT (CPI до item_nft)
#[program]
pub mod crafting {
    use super::*;

    /// Ініціалізує CraftingAuthority PDA (викликається адміністратором один раз).
    pub fn initialize_crafting_authority(
        ctx: Context<InitializeCraftingAuthority>,
    ) -> Result<()> {
        initialize_crafting_authority::handler(ctx)
    }

    /// Крафтить предмет, спалюючи ресурси та мінтячи NFT.
    pub fn craft_item(ctx: Context<CraftItem>, item_type: u8) -> Result<()> {
        craft_item::handler(ctx, item_type)
    }
}

#[cfg(test)]
mod tests {
    use crate::state::CraftingAuthority;
    use crate::constants::{RECIPES, ITEM_COUNT, RECIPE_INGREDIENTS};

    /// Перевіряє розмір акаунта CraftingAuthority
    #[test]
    fn test_crafting_authority_space() {
        // discriminator(8) + bump(1) = 9
        assert_eq!(CraftingAuthority::SPACE, 9);
    }

    /// Перевіряє рецепт Шабля козака (item_type=0)
    #[test]
    fn test_recipe_saber() {
        let recipe = RECIPES[0];
        // 3×IRON(1) + 1×WOOD(0) + 1×LEATHER(3)
        assert_eq!(recipe[0], (1, 3)); // IRON x3
        assert_eq!(recipe[1], (0, 1)); // WOOD x1
        assert_eq!(recipe[2], (3, 1)); // LEATHER x1
    }

    /// Перевіряє рецепт Посох старійшини (item_type=1)
    #[test]
    fn test_recipe_staff() {
        let recipe = RECIPES[1];
        // 2×WOOD(0) + 1×GOLD(2) + 1×DIAMOND(5)
        assert_eq!(recipe[0], (0, 2)); // WOOD x2
        assert_eq!(recipe[1], (2, 1)); // GOLD x1
        assert_eq!(recipe[2], (5, 1)); // DIAMOND x1
    }

    /// Перевіряє рецепт Броня характерника (item_type=2)
    #[test]
    fn test_recipe_armor() {
        let recipe = RECIPES[2];
        // 4×LEATHER(3) + 2×IRON(1) + 1×GOLD(2)
        assert_eq!(recipe[0], (3, 4)); // LEATHER x4
        assert_eq!(recipe[1], (1, 2)); // IRON x2
        assert_eq!(recipe[2], (2, 1)); // GOLD x1
    }

    /// Перевіряє рецепт Бойовий браслет (item_type=3)
    #[test]
    fn test_recipe_bracelet() {
        let recipe = RECIPES[3];
        // 4×IRON(1) + 2×GOLD(2) + 2×DIAMOND(5)
        assert_eq!(recipe[0], (1, 4)); // IRON x4
        assert_eq!(recipe[1], (2, 2)); // GOLD x2
        assert_eq!(recipe[2], (5, 2)); // DIAMOND x2
    }

    /// Перевіряє кількість рецептів та інгредієнтів
    #[test]
    fn test_recipe_counts() {
        assert_eq!(ITEM_COUNT, 4);
        assert_eq!(RECIPE_INGREDIENTS, 3);
        assert_eq!(RECIPES.len(), ITEM_COUNT);
        for recipe in RECIPES.iter() {
            assert_eq!(recipe.len(), RECIPE_INGREDIENTS);
            // Перевіряємо що всі resource_id в діапазоні 0-5
            for (resource_id, amount) in recipe.iter() {
                assert!(*resource_id < 6, "resource_id {} має бути < 6", resource_id);
                assert!(*amount > 0, "amount {} має бути > 0", amount);
            }
        }
    }
}
