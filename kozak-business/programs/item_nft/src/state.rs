use anchor_lang::prelude::*;

/// Singleton admin config for the `item_nft` program. Stored as a PDA
/// derived from [`crate::constants::ITEM_CONFIG_SEED`].
#[account]
#[derive(InitSpace)]
pub struct ItemConfig {
    /// The wallet authorised to perform admin actions (registering the
    /// marketplace program). Enforced via `has_one = admin`.
    pub admin: Pubkey,
    /// Program ID of the registered `marketplace` program. Only a signer PDA
    /// derived under this program's ID can satisfy the
    /// `marketplace_authority` constraint in [`crate::instructions::burn_item_nft`].
    /// `Pubkey::default()` means "not yet registered".
    pub marketplace_program: Pubkey,
    /// Canonical bump of this PDA.
    pub bump: u8,
}
