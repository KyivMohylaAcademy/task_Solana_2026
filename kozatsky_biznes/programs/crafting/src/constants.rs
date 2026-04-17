/// Рецепти крафту: [resource_id, amount] для кожного з 3 інгредієнтів, 4 предмети
///
/// Рецепти:
/// - Item 0 (Шабля козака):      3×IRON(1) + 1×WOOD(0)  + 1×LEATHER(3)
/// - Item 1 (Посох старійшини):  2×WOOD(0) + 1×GOLD(2)  + 1×DIAMOND(5)
/// - Item 2 (Броня характерника):4×LEATHER(3) + 2×IRON(1) + 1×GOLD(2)
/// - Item 3 (Бойовий браслет):   4×IRON(1) + 2×GOLD(2)  + 2×DIAMOND(5)
pub const RECIPES: [[(u8, u64); 3]; 4] = [
    [(1, 3), (0, 1), (3, 1)],  // Шабля: 3×IRON + 1×WOOD + 1×LEATHER
    [(0, 2), (2, 1), (5, 1)],  // Посох: 2×WOOD + 1×GOLD + 1×DIAMOND
    [(3, 4), (1, 2), (2, 1)],  // Броня: 4×LEATHER + 2×IRON + 1×GOLD
    [(1, 4), (2, 2), (5, 2)],  // Браслет: 4×IRON + 2×GOLD + 2×DIAMOND
];

/// Кількість предметів у грі
pub const ITEM_COUNT: usize = 4;

/// Кількість інгредієнтів у кожному рецепті
pub const RECIPE_INGREDIENTS: usize = 3;
