use anchor_lang::prelude::*;
use crate::state::{ItemNftConfig, ItemMetadata};

/// Accounts for CPI-gated ownership transfer (marketplace program only).
#[derive(Accounts)]
pub struct TransferItemOwnership<'info> {
    #[account(
        seeds = [b"caller_authority"],
        bump,
        seeds::program = config.marketplace_program,
    )]
    pub caller_authority: Signer<'info>,

    #[account(seeds = [b"item_nft_config"], bump = config.bump)]
    pub config: Account<'info, ItemNftConfig>,

    #[account(
        mut,
        seeds = [b"item_metadata", item_metadata.mint.as_ref()],
        bump = item_metadata.bump,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
}
