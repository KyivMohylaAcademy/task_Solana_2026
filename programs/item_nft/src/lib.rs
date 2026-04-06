use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::associated_token::AssociatedToken;

declare_id!("5KoKMBpkpNYQBaht7e55RzH31x7zyEXHGN19GAnz9m4s");

/// Item types corresponding to the game items
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    /// Cossack Saber: 3× Iron + 1× Wood + 1× Leather
    CossackSaber = 0,
    /// Elder Staff: 2× Wood + 1× Gold + 1× Diamond
    ElderStaff = 1,
    /// Characternik Armor: 4× Leather + 2× Iron + 1× Gold (optional)
    CharacternikArmor = 2,
    /// Battle Bracelet: 4× Iron + 2× Gold + 2× Diamond (optional)
    BattleBracelet = 3,
}

impl ItemType {
    /// Get recipe requirements: (WOOD, IRON, GOLD, LEATHER, STONE, DIAMOND)
    pub fn recipe(&self) -> [u64; 6] {
        match self {
            ItemType::CossackSaber => [1, 3, 0, 1, 0, 0],      // 1 Wood, 3 Iron, 1 Leather
            ItemType::ElderStaff => [2, 0, 1, 0, 0, 1],        // 2 Wood, 1 Gold, 1 Diamond
            ItemType::CharacternikArmor => [0, 2, 1, 4, 0, 0], // 4 Leather, 2 Iron, 1 Gold
            ItemType::BattleBracelet => [0, 4, 2, 0, 0, 2],    // 4 Iron, 2 Gold, 2 Diamond
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ItemType::CossackSaber => "Cossack Saber",
            ItemType::ElderStaff => "Elder Staff",
            ItemType::CharacternikArmor => "Characternik Armor",
            ItemType::BattleBracelet => "Battle Bracelet",
        }
    }

    pub fn symbol(&self) -> &str {
        match self {
            ItemType::CossackSaber => "SABER",
            ItemType::ElderStaff => "STAFF",
            ItemType::CharacternikArmor => "ARMOR",
            ItemType::BattleBracelet => "BRACELET",
        }
    }
}

#[program]
pub mod item_nft {
    use super::*;

    /// Initialize the item NFT config
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.bump = ctx.bumps.config;
        config.total_items_minted = 0;
        
        msg!("Item NFT manager initialized");
        Ok(())
    }

    /// Create an item NFT (called by crafting program)
    pub fn create_item(
        ctx: Context<CreateItem>,
        item_type: u8,
        uri: String,
    ) -> Result<()> {
        require!(item_type < 4, ErrorCode::InvalidItemType);
        
        let item_enum = match item_type {
            0 => ItemType::CossackSaber,
            1 => ItemType::ElderStaff,
            2 => ItemType::CharacternikArmor,
            3 => ItemType::BattleBracelet,
            _ => return Err(ErrorCode::InvalidItemType.into()),
        };

        let item_metadata = &mut ctx.accounts.item_metadata;
        item_metadata.item_type = item_type;
        item_metadata.owner = ctx.accounts.owner.key();
        item_metadata.mint = ctx.accounts.mint.key();
        item_metadata.bump = ctx.bumps.item_metadata;

        let config = &mut ctx.accounts.config;
        config.total_items_minted += 1;

        msg!("Created {} NFT for owner: {} with URI: {}", item_enum.name(), ctx.accounts.owner.key(), uri);
        Ok(())
    }

    /// Burn an item NFT (called by marketplace program)
    pub fn burn_item(ctx: Context<BurnItem>) -> Result<()> {
        msg!("Burning item NFT: {}", ctx.accounts.mint.key());
        
        // Close the item metadata account
        Ok(())
    }

    /// Transfer item ownership
    pub fn transfer_item(ctx: Context<TransferItem>) -> Result<()> {
        let item_metadata = &mut ctx.accounts.item_metadata;
        item_metadata.owner = ctx.accounts.new_owner.key();
        
        msg!("Transferred item to: {}", ctx.accounts.new_owner.key());
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + ItemConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, ItemConfig>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CreateItem<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, ItemConfig>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + ItemMetadata::INIT_SPACE,
        seeds = [b"item", mint.key().as_ref()],
        bump
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    
    /// CHECK: The NFT mint account
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    
    /// CHECK: The NFT token account
    #[account(mut)]
    pub token_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BurnItem<'info> {
    #[account(
        mut,
        seeds = [b"item", mint.key().as_ref()],
        bump = item_metadata.bump,
        close = owner
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    
    /// CHECK: The NFT mint to burn
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    
    /// CHECK: The NFT token account to burn from
    #[account(mut)]
    pub token_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub token_program: Program<'info, token::Token>,
}

#[derive(Accounts)]
pub struct TransferItem<'info> {
    #[account(
        mut,
        seeds = [b"item", mint.key().as_ref()],
        bump = item_metadata.bump,
        constraint = item_metadata.owner == owner.key() @ ErrorCode::Unauthorized
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    
    /// CHECK: The NFT mint
    pub mint: AccountInfo<'info>,
    
    pub owner: Signer<'info>,
    
    /// CHECK: The new owner
    pub new_owner: AccountInfo<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct ItemConfig {
    pub admin: Pubkey,
    pub total_items_minted: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid item type (must be 0-3)")]
    InvalidItemType,
    #[msg("Unauthorized")]
    Unauthorized,
}
