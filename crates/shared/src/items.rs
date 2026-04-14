//! Item and resource type enumerations with recipe constants.

/// The 4 craftable item types.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ItemKind {
    /// Шабля козака – Cossack Saber
    CossackSaber = 0,
    /// Посох старійшини – Elder's Staff
    ElderStaff = 1,
    /// Броня характерника – Kharakternyk Armor
    KharakternykArmor = 2,
    /// Бойовий браслет – Battle Bracelet
    BattleBracelet = 3,
}

impl ItemKind {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::CossackSaber),
            1 => Some(Self::ElderStaff),
            2 => Some(Self::KharakternykArmor),
            3 => Some(Self::BattleBracelet),
            _ => None,
        }
    }

    /// Human-readable name used in NFT metadata.
    pub fn name(self) -> &'static str {
        match self {
            Self::CossackSaber => "Шабля козака",
            Self::ElderStaff => "Посох старійшини",
            Self::KharakternykArmor => "Броня характерника",
            Self::BattleBracelet => "Бойовий браслет",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Self::CossackSaber => "SABER",
            Self::ElderStaff => "STAFF",
            Self::KharakternykArmor => "ARMOR",
            Self::BattleBracelet => "BRACE",
        }
    }
}

/// The 6 resource types (index = SPL Token-2022 kind byte).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ResourceKind {
    Wood = 0,
    Iron = 1,
    Gold = 2,
    Leather = 3,
    Stone = 4,
    Diamond = 5,
}

impl ResourceKind {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Wood),
            1 => Some(Self::Iron),
            2 => Some(Self::Gold),
            3 => Some(Self::Leather),
            4 => Some(Self::Stone),
            5 => Some(Self::Diamond),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Wood => "Дерево",
            Self::Iron => "Залізо",
            Self::Gold => "Золото",
            Self::Leather => "Шкіра",
            Self::Stone => "Камінь",
            Self::Diamond => "Алмаз",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Self::Wood => "WOOD",
            Self::Iron => "IRON",
            Self::Gold => "GOLD",
            Self::Leather => "LEATHER",
            Self::Stone => "STONE",
            Self::Diamond => "DIAMOND",
        }
    }
}

/// Crafting recipes: `RECIPES[item_kind][resource_kind] = required_amount`.
/// Index layout: [Wood, Iron, Gold, Leather, Stone, Diamond]
pub const RECIPES: [[u64; 6]; 4] = [
    [1, 3, 0, 1, 0, 0], // CossackSaber:       1 Wood + 3 Iron + 1 Leather
    [2, 0, 1, 0, 0, 1], // ElderStaff:         2 Wood + 1 Gold + 1 Diamond
    [0, 2, 1, 4, 0, 0], // KharakternykArmor:  4 Leather + 2 Iron + 1 Gold
    [0, 4, 2, 0, 0, 2], // BattleBracelet:     4 Iron + 2 Gold + 2 Diamond
];
