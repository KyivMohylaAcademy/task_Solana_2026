use anchor_lang::prelude::*;
use crate::state::Listing;

/// List an item for sale on the marketplace
pub fn list_item(
    ctx: Context<ListItem>,
    price: u64,
) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    listing.seller = ctx.accounts.seller.key();
    listing.item_mint = ctx.accounts.item_mint.key();
    listing.price = price;
    listing.bump = ctx.bumps.listing;
    
    emit!(ItemListed {
        seller: ctx.accounts.seller.key(),
        item_mint: ctx.accounts.item_mint.key(),
        price,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

#[derive(Accounts)]
pub struct ListItem<'info> {
    #[account(
        init,
        payer = seller,
        space = Listing::SPACE,
        seeds = [b"listing", item_mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,
    
    pub item_mint: Signer<'info>,
    
    #[account(mut)]
    pub seller: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[event]
pub struct ItemListed {
    pub seller: Pubkey,
    pub item_mint: Pubkey,
    pub price: u64,
    pub timestamp: i64,
}
