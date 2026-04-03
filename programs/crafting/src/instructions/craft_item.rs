use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use crate::state::{ItemMetadata, CraftingRecipe};
use crate::errors::CraftingError;

/// Craft an item by burning resources
pub fn craft_item(
    ctx: Context<CraftItem>,
    item_type: u8,
) -> Result<()> {
    // Validate item type
    let recipe = CraftingRecipe::get_recipe(item_type)
        .ok_or(CraftingError::InvalidItemType)?;
    
    // Store item metadata
    let item_metadata = &mut ctx.accounts.item_metadata;
    item_metadata.item_type = item_type;
    item_metadata.owner = ctx.accounts.owner.key();
    item_metadata.mint = ctx.accounts.item_mint.key();
    item_metadata.bump = ctx.bumps.item_metadata;
    
    // Note: Actual resource burning and NFT minting would be done here via CPI
    // For now, this represents the core crafting logic
    
    emit!(ItemCrafted {
        owner: ctx.accounts.owner.key(),
        item_type,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CraftItem<'info> {
    #[account(
        init,
        payer = owner,
        space = ItemMetadata::SPACE,
        seeds = [b"item_metadata", item_mint.key().as_ref()],
        bump
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    
    /// The NFT mint for the crafted item
    pub item_mint: Signer<'info>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct ItemCrafted {
    pub owner: Pubkey,
    pub item_type: u8,
    pub timestamp: i64,
}
