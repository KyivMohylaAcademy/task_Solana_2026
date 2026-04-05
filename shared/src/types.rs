//! Shared enums and metadata descriptors for every resource and item type.

use anchor_lang::prelude::*;

use crate::constants::{ITEM_COUNT, RESOURCE_COUNT};
use crate::errors::GameErrorCode;

/// Human-readable off-chain metadata for an item NFT type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemDescriptor {
    /// Display name used in NFT metadata.
    pub name: &'static str,
    /// Short ticker-like symbol used in NFT metadata.
    pub symbol: &'static str,
    /// Metadata URI embedded into the minted NFT.
    pub uri: &'static str,
}

/// Enumerates all base resources that can be minted through `search`.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ResourceType {
    Wood = 0,
    Iron = 1,
    Gold = 2,
    Leather = 3,
    Stone = 4,
    Diamond = 5,
}

impl ResourceType {
    /// Ordered list of all supported resource types.
    pub const ALL: [Self; RESOURCE_COUNT] = [
        Self::Wood,
        Self::Iron,
        Self::Gold,
        Self::Leather,
        Self::Stone,
        Self::Diamond,
    ];

    /// Converts a zero-based integer into a validated resource enum.
    pub fn from_index(index: usize) -> Result<Self> {
        Self::ALL
            .get(index)
            .copied()
            .ok_or_else(|| error!(GameErrorCode::InvalidResourceTypeIndex))
    }

    /// Returns the zero-based index used in arrays and PDA derivations.
    pub const fn as_index(self) -> usize {
        self as usize
    }
}

/// Enumerates all craftable NFT item types supported by the game.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ItemType {
    KozakSabre = 0,
    ElderStaff = 1,
    CharacteristicArmor = 2,
    BattleBracelet = 3,
}

impl ItemType {
    /// Ordered list of all supported craftable item types.
    pub const ALL: [Self; ITEM_COUNT] = [
        Self::KozakSabre,
        Self::ElderStaff,
        Self::CharacteristicArmor,
        Self::BattleBracelet,
    ];

    /// Converts a zero-based integer into a validated item enum.
    pub fn from_index(index: usize) -> Result<Self> {
        Self::ALL
            .get(index)
            .copied()
            .ok_or_else(|| error!(GameErrorCode::InvalidItemTypeIndex))
    }

    /// Returns the zero-based index used in arrays and price tables.
    pub const fn as_index(self) -> usize {
        self as usize
    }

    /// Returns the static human-readable descriptor for this item type.
    pub const fn descriptor(self) -> &'static ItemDescriptor {
        &ITEM_DESCRIPTORS[self.as_index()]
    }
}

/// Static descriptors used for NFT metadata URIs, names and symbols.
pub const ITEM_DESCRIPTORS: [ItemDescriptor; ITEM_COUNT] = [
    ItemDescriptor {
        name: "Kozak Sabre",
        symbol: "SABRE",
        uri: "https://example.com/items/kozak-sabre.json",
    },
    ItemDescriptor {
        name: "Elder Staff",
        symbol: "STAFF",
        uri: "https://example.com/items/elder-staff.json",
    },
    ItemDescriptor {
        name: "Characteristic Armor",
        symbol: "ARMOR",
        uri: "https://example.com/items/characteristic-armor.json",
    },
    ItemDescriptor {
        name: "Battle Bracelet",
        symbol: "BRACE",
        uri: "https://example.com/items/battle-bracelet.json",
    },
];
