use anchor_lang::prelude::*;

/// Item metadata stored on-chain
#[account]
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

impl ItemMetadata {
    pub const SPACE: usize = 8 + 1 + 32 + 32 + 1;
}

/// Item type enumeration
pub enum ItemType {
    CossackSabre = 0,      // 3× Iron + 1× Wood + 1× Leather
    ElderStaff = 1,        // 2× Wood + 1× Gold + 1× Diamond
    Armor = 2,             // 4× Leather + 2× Iron + 1× Gold (optional)
    BattleBracelet = 3,    // 4× Iron + 2× Gold + 2× Diamond (optional)
}

/// Resource requirements for each item
pub struct CraftingRecipe {
    pub item_type: u8,
    pub resources: [u8; 6], // amounts for each resource type
}

impl CraftingRecipe {
    pub fn get_recipe(item_type: u8) -> Option<Self> {
        match item_type {
            0 => Some(CraftingRecipe {
                item_type: 0,
                resources: [1, 3, 0, 1, 0, 0], // 1 Wood, 3 Iron, 1 Leather
            }),
            1 => Some(CraftingRecipe {
                item_type: 1,
                resources: [2, 0, 1, 0, 0, 1], // 2 Wood, 1 Gold, 1 Diamond
            }),
            2 => Some(CraftingRecipe {
                item_type: 2,
                resources: [0, 2, 1, 4, 0, 0], // 2 Iron, 1 Gold, 4 Leather
            }),
            3 => Some(CraftingRecipe {
                item_type: 3,
                resources: [0, 4, 2, 0, 0, 2], // 4 Iron, 2 Gold, 2 Diamond
            }),
            _ => None,
        }
    }
}
