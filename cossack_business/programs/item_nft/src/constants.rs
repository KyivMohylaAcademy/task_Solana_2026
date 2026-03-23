use anchor_lang::prelude::*;

pub const ITEM_COUNT: usize = 4;

pub const ITEM_NAMES: [&str; ITEM_COUNT] = [
    "Cossack Saber",
    "Elder Staff",
    "Mage Armor",
    "Battle Bracelet",
];

pub const ITEM_SYMBOLS: [&str; ITEM_COUNT] = ["SABER", "STAFF", "ARMOR", "BRACLT"];

/// Metaplex Token Metadata program ID
pub const MPL_TOKEN_METADATA_ID: Pubkey =
    anchor_lang::pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
