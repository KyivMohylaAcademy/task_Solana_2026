/// Кількість ресурсів у грі.
pub const RESOURCE_COUNT: u8 = 6;

/// PDA seed для GameConfig.
pub const GAME_CONFIG_SEED: &[u8] = b"game_config";

/// PDA seed для mint authority.
pub const MINT_AUTHORITY_SEED: &[u8] = b"mint_authority";

/// Назви ресурсів: [WOOD, IRON, GOLD, LEATHER, STONE, DIAMOND].
pub const RESOURCE_NAMES: [&str; 6] = [
    "Дерево", "Залізо", "Золото", "Шкіра", "Камінь", "Алмаз",
];

/// Символи ресурсів.
pub const RESOURCE_SYMBOLS: [&str; 6] = [
    "WOOD", "IRON", "GOLD", "LEATHER", "STONE", "DIAMOND",
];
