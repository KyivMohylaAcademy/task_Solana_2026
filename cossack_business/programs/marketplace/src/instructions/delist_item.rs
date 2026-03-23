use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::Listing;
use crate::errors::MarketError;

/// Accounts for cancelling a listing and returning the NFT to the seller.
#[derive(Accounts)]
pub struct DelistItem<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        mut,
        seeds = [b"listing", listing.item_mint.as_ref()],
        bump = listing.bump,
        constraint = listing.seller == seller.key() @ MarketError::NotOwner,
        close = seller,
    )]
    pub listing: Account<'info, Listing>,

    pub nft_mint: Account<'info, Mint>,

    /// CHECK: Escrow authority PDA
    #[account(
        seeds = [b"escrow", nft_mint.key().as_ref()],
        bump = listing.escrow_bump,
    )]
    pub escrow_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = escrow_authority,
    )]
    pub escrow_nft_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = seller,
    )]
    pub seller_nft_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
