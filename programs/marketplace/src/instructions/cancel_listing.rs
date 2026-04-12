use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
    TransferChecked,
};

use crate::errors::MarketplaceError;
use crate::state::Listing;

use super::list_item::{ESCROW_SEED, LISTING_SEED};

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    /// Listing PDA — закривається, lamports повертаються продавцю.
    #[account(
        mut,
        seeds = [LISTING_SEED, nft_mint.key().as_ref()],
        bump = listing.bump,
        constraint = listing.seller == seller.key() @ MarketplaceError::NotSeller,
        close = seller,
    )]
    pub listing: Account<'info, Listing>,

    /// NFT мінт.
    pub nft_mint: InterfaceAccount<'info, Mint>,

    /// Ескроу токен-акаунт — закривається.
    #[account(
        mut,
        seeds = [ESCROW_SEED, nft_mint.key().as_ref()],
        bump,
        token::mint = nft_mint,
        token::authority = listing,
        token::token_program = token_program,
    )]
    pub escrow_account: InterfaceAccount<'info, TokenAccount>,

    /// Токен-акаунт продавця — повернення NFT.
    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = seller,
        token::token_program = token_program,
    )]
    pub seller_nft_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<CancelListing>) -> Result<()> {
    let listing_bump = ctx.accounts.listing.bump;
    let nft_mint_key = ctx.accounts.nft_mint.key();
    let seeds: &[&[u8]] = &[LISTING_SEED, nft_mint_key.as_ref(), &[listing_bump]];
    let signer_seeds = &[seeds];

    // Повертаємо NFT продавцю
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.escrow_account.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.seller_nft_account.to_account_info(),
                authority: ctx.accounts.listing.to_account_info(),
            },
            signer_seeds,
        ),
        1,
        0,
    )?;

    // Закриваємо ескроу
    close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        CloseAccount {
            account: ctx.accounts.escrow_account.to_account_info(),
            destination: ctx.accounts.seller.to_account_info(),
            authority: ctx.accounts.listing.to_account_info(),
        },
        signer_seeds,
    ))?;

    msg!("Лістинг скасовано. NFT повернено продавцю.");
    Ok(())
}
