/// Рецепти крафту. Кожен рядок — [resource_id, amount, resource_id, amount, ...].
/// Формат: [(resource_id, amount); MAX_INGREDIENTS].
/// Padding нулями якщо інгредієнтів менше.
///
/// ID ресурсів: 0=WOOD, 1=IRON, 2=GOLD, 3=LEATHER, 4=STONE, 5=DIAMOND

pub const MAX_INGREDIENTS: usize = 4;

/// Кількість різних типів предметів.
pub const ITEM_TYPE_COUNT: u8 = 4;

/// Рецепти: (resource_id, amount) пари.
/// Item 0 — Шабля козака: 3×IRON + 1×WOOD + 1×LEATHER
/// Item 1 — Посох старійшини: 2×WOOD + 1×GOLD + 1×DIAMOND
/// Item 2 — Броня характерника: 4×LEATHER + 2×IRON + 1×GOLD
/// Item 3 — Бойовий браслет: 4×IRON + 2×GOLD + 2×DIAMOND
pub const RECIPES: [[(u8, u64); MAX_INGREDIENTS]; 4] = [
    // Item 0: Шабля козака
    [(1, 3), (0, 1), (3, 1), (0, 0)],
    // Item 1: Посох старійшини
    [(0, 2), (2, 1), (5, 1), (0, 0)],
    // Item 2: Броня характерника
    [(3, 4), (1, 2), (2, 1), (0, 0)],
    // Item 3: Бойовий браслет
    [(1, 4), (2, 2), (5, 2), (0, 0)],
];
