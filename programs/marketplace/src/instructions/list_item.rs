use anchor_lang::prelude::*;
use anchor_spl::token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::errors::MarketplaceError;
use crate::state::Listing;

pub const LISTING_SEED: &[u8] = b"listing";
pub const ESCROW_SEED: &[u8] = b"escrow";

#[derive(Accounts)]
#[instruction(price: u64)]
pub struct ListItem<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    /// Listing PDA.
    #[account(
        init,
        payer = seller,
        space = Listing::LEN,
        seeds = [LISTING_SEED, nft_mint.key().as_ref()],
        bump,
    )]
    pub listing: Account<'info, Listing>,

    /// NFT мінт.
    pub nft_mint: InterfaceAccount<'info, Mint>,

    /// Токен-акаунт продавця з NFT.
    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = seller,
        token::token_program = token_program,
    )]
    pub seller_nft_account: InterfaceAccount<'info, TokenAccount>,

    /// Ескроу токен-акаунт (PDA authority = listing PDA).
    #[account(
        init,
        payer = seller,
        seeds = [ESCROW_SEED, nft_mint.key().as_ref()],
        bump,
        token::mint = nft_mint,
        token::authority = listing,
        token::token_program = token_program,
    )]
    pub escrow_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ListItem>, price: u64) -> Result<()> {
    require!(price > 0, MarketplaceError::ZeroPrice);

    let listing = &mut ctx.accounts.listing;
    listing.seller = ctx.accounts.seller.key();
    listing.nft_mint = ctx.accounts.nft_mint.key();
    listing.price = price;
    listing.bump = ctx.bumps.listing;

    // Переводимо NFT в ескроу
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.seller_nft_account.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.escrow_account.to_account_info(),
                authority: ctx.accounts.seller.to_account_info(),
            },
        ),
        1,
        0, // decimals
    )?;

    msg!("NFT {} виставлено за {} MagicToken", ctx.accounts.nft_mint.key(), price);
    Ok(())
}
