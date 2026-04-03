use anchor_lang::prelude::*;
use crate::state::{ItemNFTMetadata, ITEM_NAMES, ITEM_DESCRIPTIONS};
use crate::errors::ItemNFTError;

/// Create a new item NFT
pub fn create_item_nft(
    ctx: Context<CreateItemNFT>,
    item_type: u8,
    uri: String,
) -> Result<()> {
    if item_type >= 4 {
        return Err(ItemNFTError::InvalidItemType.into());
    }
    
    let nft_metadata = &mut ctx.accounts.nft_metadata;
    nft_metadata.item_type = item_type;
    nft_metadata.creator = ctx.accounts.creator.key();
    nft_metadata.mint = ctx.accounts.mint.key();
    nft_metadata.uri = uri.clone();
    nft_metadata.bump = ctx.bumps.nft_metadata;
    
    emit!(ItemNFTCreated {
        creator: ctx.accounts.creator.key(),
        mint: ctx.accounts.mint.key(),
        item_type,
        uri,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CreateItemNFT<'info> {
    #[account(
        init,
        payer = creator,
        space = ItemNFTMetadata::SPACE,
        seeds = [b"item_nft", mint.key().as_ref()],
        bump
    )]
    pub nft_metadata: Account<'info, ItemNFTMetadata>,
    
    pub mint: Signer<'info>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[event]
pub struct ItemNFTCreated {
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub item_type: u8,
    pub uri: String,
    pub timestamp: i64,
}
