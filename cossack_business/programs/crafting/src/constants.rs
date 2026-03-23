pub const RESOURCE_COUNT: usize = 6;

/// Crafting recipes: each row is [WOOD, IRON, GOLD, LEATHER, STONE, DIAMOND]
pub const RECIPES: [[u8; RESOURCE_COUNT]; 4] = [
    [1, 3, 0, 1, 0, 0], // Cossack Saber
    [2, 0, 1, 0, 0, 1], // Elder Staff
    [0, 2, 1, 4, 0, 0], // Mage Armor
    [0, 4, 2, 0, 0, 2], // Battle Bracelet
];
