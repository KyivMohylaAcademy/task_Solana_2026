use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::token_interface::{
    Mint as MintInterface, TokenAccount as TokenAccountInterface, TokenInterface,
};
use item_nft::program::ItemNft;
use crate::state::Listing;
use crate::errors::MarketError;

/// Accounts for buying a listed item (MagicToken to seller, NFT from escrow to buyer).
#[derive(Accounts)]
pub struct BuyItem<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: This program's CPI authority PDA
    #[account(seeds = [b"caller_authority"], bump)]
    pub caller_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"listing", listing.item_mint.as_ref()],
        bump = listing.bump,
        close = seller,
    )]
    pub listing: Box<Account<'info, Listing>>,

    /// CHECK: Validated via listing.seller
    #[account(
        mut,
        constraint = seller.key() == listing.seller @ MarketError::SellerMismatch,
    )]
    pub seller: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"item_metadata", listing.item_mint.as_ref()],
        bump,
        seeds::program = item_nft::ID,
    )]
    pub item_metadata: Box<Account<'info, item_nft::ItemMetadata>>,

    #[account(
        seeds = [b"item_nft_config"],
        bump = item_nft_config.bump,
        seeds::program = item_nft_program.key(),
    )]
    pub item_nft_config: Box<Account<'info, item_nft::ItemNftConfig>>,

    #[account(
        constraint = nft_mint.key() == listing.item_mint @ MarketError::MintMismatch,
    )]
    pub nft_mint: Box<Account<'info, Mint>>,

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
    pub escrow_nft_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = buyer,
    )]
    pub buyer_nft_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"magic_config"],
        bump = magic_token_config.bump,
        seeds::program = magic_token::ID,
    )]
    pub magic_token_config: Box<Account<'info, magic_token::MagicTokenConfig>>,

    #[account(
        constraint = magic_token_mint.key() == magic_token_config.mint @ MarketError::MintMismatch,
    )]
    pub magic_token_mint: Box<InterfaceAccount<'info, MintInterface>>,

    #[account(mut)]
    pub buyer_magic_ata: Box<InterfaceAccount<'info, TokenAccountInterface>>,

    #[account(mut)]
    pub seller_magic_ata: Box<InterfaceAccount<'info, TokenAccountInterface>>,

    pub item_nft_program: Program<'info, ItemNft>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
