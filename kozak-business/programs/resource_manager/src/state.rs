use anchor_lang::prelude::*;

use crate::constants::RESOURCE_COUNT;

/// Global game settings. Stored as a singleton PDA derived from
/// [`crate::GAME_CONFIG_SEED`].
///
/// `#[derive(InitSpace)]` auto-calculates `GameConfig::INIT_SPACE` — the
/// byte cost of the fields — so we never count bytes by hand.
#[account]
#[derive(InitSpace)]
pub struct GameConfig {
    /// The wallet authorised to perform admin actions such as initialising
    /// the resource mints. Enforced on subsequent instructions via the
    /// `has_one = admin` Anchor constraint.
    pub admin: Pubkey,
    /// Mint addresses of the 6 base resources, indexed by `resource_id`.
    /// Zeroed at creation; populated one at a time by
    /// [`crate::instructions::initialize_resource_mint`]. A `Pubkey::default()`
    /// entry means "not yet initialised".
    pub resource_mints: [Pubkey; RESOURCE_COUNT],
    /// Program ID of the registered `search` program. Set by the admin via
    /// [`crate::instructions::set_search_program`] after deployment. This
    /// address is the only one whose `[SEARCH_AUTHORITY_SEED]` PDA can sign
    /// the gated [`crate::instructions::mint_resource`] CPI — that's how we
    /// stop random programs from minting our resources.
    /// `Pubkey::default()` means "not yet registered".
    pub search_program: Pubkey,
    /// Canonical bump of this PDA. Storing it lets callers sign as this PDA
    /// without re-deriving the bump each time.
    pub bump: u8,
}
