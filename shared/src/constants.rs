//! Shared gameplay constants used by both on-chain programs and off-chain scripts.

/// Number of base Token-2022 resource mints available in the game.
pub const RESOURCE_COUNT: usize = 6;
/// Number of craftable NFT item types supported in v1.
pub const ITEM_COUNT: usize = 4;
/// Required delay between two successful search actions for the same player.
pub const SEARCH_COOLDOWN_SECONDS: i64 = 60;
