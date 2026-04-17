/// Кількість базових ресурсів у грі
pub const RESOURCE_COUNT: usize = 6;

/// Назви ресурсів (українська)
pub const RESOURCE_NAMES: [&str; RESOURCE_COUNT] = [
    "Дерево",
    "Залізо",
    "Золото",
    "Шкіра",
    "Камінь",
    "Алмаз",
];

/// Символи ресурсів
pub const RESOURCE_SYMBOLS: [&str; RESOURCE_COUNT] = [
    "WOOD", "IRON", "GOLD", "LEATHER", "STONE", "DIAMOND",
];

/// Таймаут пошуку ресурсів у секундах
pub const SEARCH_COOLDOWN_SECONDS: i64 = 60;

/// Кількість ресурсів, що генеруються за один пошук
pub const RESOURCES_PER_SEARCH: u64 = 1;