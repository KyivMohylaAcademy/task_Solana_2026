/// Recipe burn amounts indexed by [item_type][resource_id].
/// resource_ids: 0=Wood, 1=Iron, 2=Gold, 3=Leather, 4=Stone, 5=Diamond
pub const RECIPES: [[u64; 6]; 4] = [
    // item_type 0: Cossack Saber — 1×Wood + 3×Iron + 1×Leather
    [1, 3, 0, 1, 0, 0],
    // item_type 1: Elder's Staff — 2×Wood + 1×Gold + 1×Diamond
    [2, 0, 1, 0, 0, 1],
    // item_type 2: Kharakternyk's Armor — 2×Iron + 1×Gold + 4×Leather
    [0, 2, 1, 4, 0, 0],
    // item_type 3: Battle Bracelet — 4×Iron + 2×Gold + 2×Diamond
    [0, 4, 2, 0, 0, 2],
];
