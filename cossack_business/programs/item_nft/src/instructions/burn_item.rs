use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::{ItemNftConfig, ItemMetadata};
use crate::errors::ItemError;
use crate::constants::MPL_TOKEN_METADATA_ID;

/// Accounts for CPI-gated NFT burning (marketplace program only).
#[derive(Accounts)]
pub struct BurnItem<'info> {
    #[account(
        seeds = [b"caller_authority"],
        bump,
        seeds::program = config.marketplace_program,
    )]
    pub caller_authority: Signer<'info>,

    #[account(seeds = [b"item_nft_config"], bump = config.bump)]
    pub config: Account<'info, ItemNftConfig>,

    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        close = player,
        seeds = [b"item_metadata", nft_mint.key().as_ref()],
        bump = item_metadata.bump,
        constraint = item_metadata.owner == player.key() @ ItemError::NotOwner,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = player,
    )]
    pub player_nft_ata: Account<'info, TokenAccount>,

    /// CHECK: Metaplex metadata (validated by CPI)
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK: Metaplex master edition (validated by CPI)
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: Metaplex Token Metadata program
    #[account(address = MPL_TOKEN_METADATA_ID)]
    pub metadata_program: UncheckedAccount<'info>,

    /// CHECK: Sysvar instructions account for Metaplex burn
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub sysvar_instructions: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
