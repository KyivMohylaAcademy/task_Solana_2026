use anchor_lang::prelude::*;

/// Configuration PDA for the MagicToken program, storing the mint address
/// and the authorized marketplace program for CPI-gated minting.
#[account]
#[derive(InitSpace)]
pub struct MagicTokenConfig {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub marketplace_program: Pubkey,
    pub bump: u8,
    pub mint_authority_bump: u8,
}
