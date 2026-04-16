use anchor_lang::prelude::*;

/// PDA seed prefix for [`crate::state::Player`] accounts. Full seeds are
/// `[PLAYER_SEED, wallet.key().as_ref()]`, so each wallet has exactly one
/// player account at a deterministic address.
#[constant]
pub const PLAYER_SEED: &[u8] = b"player";

/// Minimum number of seconds between two `search_resources` calls for the
/// same player. Enforced via the `Clock` sysvar.
pub const SEARCH_COOLDOWN_SECONDS: i64 = 60;

/// Number of resource units minted per `search_resources` call. Each unit is
/// independently randomised, so a single search can yield up to
/// `RESOURCES_PER_SEARCH` distinct resources.
pub const RESOURCES_PER_SEARCH: usize = 3;
