use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use crate::state::Listing;
use crate::errors::MarketplaceError;

/// Buy an item from the marketplace
pub fn buy_item(
    ctx: Context<BuyItem>,
) -> Result<()> {
    let listing = &ctx.accounts.listing;
    
    // Validate price
    if listing.price == 0 {
        return Err(MarketplaceError::InvalidPrice.into());
    }
    
    // Note: Actual token transfer would happen here via CPI
    // This includes:
    // 1. Transfer MagicToken from buyer to seller
    // 2. Mint MagicToken from marketplace if needed
    // 3. Burn the NFT item
    
    emit!(ItemSold {
        seller: listing.seller,
        buyer: ctx.accounts.buyer.key(),
        item_mint: listing.item_mint,
        price: listing.price,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

#[derive(Accounts)]
pub struct BuyItem<'info> {
    #[account(
        mut,
        close = buyer,
    )]
    pub listing: Account<'info, Listing>,
    
    pub item_mint: Signer<'info>,
    
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct ItemSold {
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub item_mint: Pubkey,
    pub price: u64,
    pub timestamp: i64,
}
