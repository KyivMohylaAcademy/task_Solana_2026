use anchor_lang::prelude::*;

pub const RESOURCE_COUNT: usize = 6;
pub const ITEM_COUNT: usize = 4;

pub const GAME_CONFIG_SEED: &[u8] = b"game-config";
pub const PLAYER_SEED: &[u8] = b"player";
pub const RESOURCE_MINT_SEED: &[u8] = b"resource-mint";
pub const RESOURCE_AUTHORITY_SEED: &[u8] = b"resource-authority";
pub const ITEM_MINT_SEED: &[u8] = b"item-mint";
pub const ITEM_METADATA_SEED: &[u8] = b"item-metadata";
pub const ITEM_AUTHORITY_SEED: &[u8] = b"item-authority";
pub const CRAFTING_AUTHORITY_SEED: &[u8] = b"crafting-authority";
pub const SEARCH_AUTHORITY_SEED: &[u8] = b"search-authority";
pub const MARKETPLACE_AUTHORITY_SEED: &[u8] = b"marketplace-authority";
pub const MAGIC_MINT_SEED: &[u8] = b"magic-mint";
pub const MAGIC_AUTHORITY_SEED: &[u8] = b"magic-authority";

pub const RESOURCE_MANAGER_ID_STR: &str = "BnswUmgoVYBc4kkVbGethzDsAoRE4bGX3p19BJ4RuU43";
pub const ITEM_NFT_ID_STR: &str = "6ZFgUpi36moUoWHokvurbZfBY7wuG4tf28WkJR3d6EZP";
pub const CRAFTING_ID_STR: &str = "EZdAg3bGtT4FwK9xcpUKM6UuJzYB8BMvXyKoHz3mS986";
pub const SEARCH_ID_STR: &str = "7yPJgKSZYcUCPgrEBmcQ7z86Frz57H6bsU5hBycStgp9";
pub const MARKETPLACE_ID_STR: &str = "E1nMz6JbstqDK9cEFhx1g3XrAJK8J2d9kvGiZTdYVaK9";
pub const MAGIC_TOKEN_ID_STR: &str = "D6TYLNDSrga9igvU5NwHwjgYtxyeTvLNPXGB9fF5p1PB";

pub const DEFAULT_ITEM_PRICES: [u64; ITEM_COUNT] = [25, 45, 60, 90];

pub const SABER_RECIPE: [u64; RESOURCE_COUNT] = [1, 3, 0, 1, 0, 0];
pub const STAFF_RECIPE: [u64; RESOURCE_COUNT] = [2, 0, 1, 0, 0, 1];
pub const ARMOR_RECIPE: [u64; RESOURCE_COUNT] = [0, 2, 1, 4, 0, 0];
pub const BRACELET_RECIPE: [u64; RESOURCE_COUNT] = [0, 4, 2, 0, 0, 2];

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
#[repr(u8)]
pub enum ResourceKind {
    Wood = 0,
    Iron = 1,
    Gold = 2,
    Leather = 3,
    Stone = 4,
    Diamond = 5,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
#[repr(u8)]
pub enum ItemKind {
    Saber = 0,
    Staff = 1,
    Armor = 2,
    Bracelet = 3,
}

pub fn recipe_for(item_type: u8) -> Option<[u64; RESOURCE_COUNT]> {
    match item_type {
        0 => Some(SABER_RECIPE),
        1 => Some(STAFF_RECIPE),
        2 => Some(ARMOR_RECIPE),
        3 => Some(BRACELET_RECIPE),
        _ => None,
    }
}

pub fn resource_manager_id() -> Pubkey {
    RESOURCE_MANAGER_ID_STR
        .parse()
        .expect("resource manager id")
}

pub fn item_nft_id() -> Pubkey {
    ITEM_NFT_ID_STR.parse().expect("item nft id")
}

pub fn crafting_id() -> Pubkey {
    CRAFTING_ID_STR.parse().expect("crafting id")
}

pub fn search_id() -> Pubkey {
    SEARCH_ID_STR.parse().expect("search id")
}

pub fn marketplace_id() -> Pubkey {
    MARKETPLACE_ID_STR.parse().expect("marketplace id")
}

pub fn magic_token_id() -> Pubkey {
    MAGIC_TOKEN_ID_STR.parse().expect("magic token id")
}

pub fn item_name(item_type: u8) -> Option<&'static str> {
    match item_type {
        0 => Some("Shablya Kozaka"),
        1 => Some("Posokh Starishiyny"),
        2 => Some("Bronya Kharakternyka"),
        3 => Some("Boyovyi Braslet"),
        _ => None,
    }
}

pub fn item_symbol(item_type: u8) -> Option<&'static str> {
    match item_type {
        0 => Some("SABER"),
        1 => Some("STAFF"),
        2 => Some("ARMOR"),
        3 => Some("BRACE"),
        _ => None,
    }
}

pub fn item_price(item_type: u8, configured_prices: &[u64; ITEM_COUNT]) -> Option<u64> {
    configured_prices.get(item_type as usize).copied()
}

pub fn resource_name(resource_id: u8) -> Option<&'static str> {
    match resource_id {
        0 => Some("Wood"),
        1 => Some("Iron"),
        2 => Some("Gold"),
        3 => Some("Leather"),
        4 => Some("Stone"),
        5 => Some("Diamond"),
        _ => None,
    }
}

pub fn resource_symbol(resource_id: u8) -> Option<&'static str> {
    match resource_id {
        0 => Some("WOOD"),
        1 => Some("IRON"),
        2 => Some("GOLD"),
        3 => Some("LEATHER"),
        4 => Some("STONE"),
        5 => Some("DIAMOND"),
        _ => None,
    }
}
