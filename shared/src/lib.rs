//! Shared constants, types and PDA helpers reused across every on-chain program.

/// Shared numeric constants describing the game economy.
pub mod constants;
/// Shared Anchor error codes used across the program suite.
pub mod errors;
/// Shared crafting recipes for all supported item types.
pub mod recipes;
/// Shared PDA seeds and canonical program identifiers.
pub mod seeds;
/// Shared enums and metadata descriptors for resources and items.
pub mod types;

use anchor_lang::prelude::*;

/// Re-exports the shared numeric constants for downstream programs.
pub use constants::{ITEM_COUNT, RESOURCE_COUNT, SEARCH_COOLDOWN_SECONDS};
/// Re-exports the shared error enum used in program constraints.
pub use errors::GameErrorCode;
/// Re-exports the canonical crafting recipe table and lookup helpers.
pub use recipes::{recipe_for, validate_recipe_table, Recipe, RECIPES};
/// Re-exports PDA helpers and well-known program IDs.
pub use seeds::*;
/// Re-exports core resource and item discriminants.
pub use types::{ItemType, ResourceType};

/// Verifies shared bootstrap assumptions that every program depends on.
pub fn validate_bootstrap_config() -> Result<()> {
    validate_recipe_table()
}
