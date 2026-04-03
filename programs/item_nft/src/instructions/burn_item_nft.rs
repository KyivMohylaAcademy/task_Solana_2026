use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use crate::state::ItemNFTMetadata;

/// Burn an item NFT (only from Marketplace)
pub fn burn_item_nft(
    ctx: Context<BurnItemNFT>,
) -> Result<()> {
    emit!(ItemNFTBurned {
        mint: ctx.accounts.nft_metadata.mint,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

#[derive(Accounts)]
pub struct BurnItemNFT<'info> {
    #[account(
        mut,
        close = owner,
    )]
    pub nft_metadata: Account<'info, ItemNFTMetadata>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub token_2022_program: Program<'info, Token2022>,
}

#[event]
pub struct ItemNFTBurned {
    pub mint: Pubkey,
    pub timestamp: i64,
}
