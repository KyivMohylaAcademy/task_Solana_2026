//! Canonical crafting recipes shared by on-chain validation and off-chain tooling.

use anchor_lang::prelude::*;

use crate::constants::{ITEM_COUNT, RESOURCE_COUNT};
use crate::errors::GameErrorCode;
use crate::types::{ItemType, ResourceType};

/// Static crafting recipe describing how many resources are required for one item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    /// The resulting item type produced by the recipe.
    pub item_type: ItemType,
    /// Per-resource costs indexed by `ResourceType::as_index()`.
    pub costs: [u64; RESOURCE_COUNT],
}

impl Recipe {
    /// Returns the required amount for a specific resource inside this recipe.
    pub const fn requires(&self, resource_type: ResourceType) -> u64 {
        self.costs[resource_type as usize]
    }
}

/// Canonical recipe table used by on-chain crafting validation.
pub const RECIPES: [Recipe; ITEM_COUNT] = [
    Recipe {
        item_type: ItemType::KozakSabre,
        costs: [1, 3, 0, 1, 0, 0],
    },
    Recipe {
        item_type: ItemType::ElderStaff,
        costs: [2, 0, 1, 0, 0, 1],
    },
    Recipe {
        item_type: ItemType::CharacteristicArmor,
        costs: [0, 2, 1, 4, 0, 0],
    },
    Recipe {
        item_type: ItemType::BattleBracelet,
        costs: [0, 4, 2, 0, 0, 2],
    },
];

/// Returns the canonical recipe for the requested item type.
pub fn recipe_for(item_type: ItemType) -> &'static Recipe {
    &RECIPES[item_type.as_index()]
}

/// Validates that the static recipe table is internally consistent.
pub fn validate_recipe_table() -> Result<()> {
    for (expected_item_index, recipe) in RECIPES.iter().enumerate() {
        require!(
            recipe.item_type.as_index() == expected_item_index,
            GameErrorCode::RecipeTableOutOfSync
        );
        require!(
            recipe.costs.iter().any(|amount| *amount > 0),
            GameErrorCode::RecipeMustRequireAtLeastOneResource
        );
    }

    Ok(())
}
