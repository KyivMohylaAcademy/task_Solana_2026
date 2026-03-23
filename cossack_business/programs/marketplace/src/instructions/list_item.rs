use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::Listing;
use crate::errors::MarketError;

/// Accounts for listing an NFT on the marketplace with escrow.
#[derive(Accounts)]
pub struct ListItem<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        constraint = item_metadata.owner == seller.key() @ MarketError::NotOwner,
    )]
    pub item_metadata: Account<'info, item_nft::ItemMetadata>,

    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = seller,
    )]
    pub seller_nft_ata: Account<'info, TokenAccount>,

    /// CHECK: Escrow authority PDA
    #[account(
        seeds = [b"escrow", nft_mint.key().as_ref()],
        bump,
    )]
    pub escrow_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = seller,
        associated_token::mint = nft_mint,
        associated_token::authority = escrow_authority,
    )]
    pub escrow_nft_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = seller,
        space = 8 + Listing::INIT_SPACE,
        seeds = [b"listing", nft_mint.key().as_ref()],
        bump,
    )]
    pub listing: Account<'info, Listing>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
