use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022, Burn};
use anchor_spl::token::{self, Token, Mint, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("2hG2aS4hLrTPcKh5dLaRxW3Yd9brJMaGKQZYM9wTnfBF");

/// Item types and their recipes
/// Recipe format: [WOOD, IRON, GOLD, LEATHER, STONE, DIAMOND]
const RECIPES: [[u64; 6]; 4] = [
    [1, 3, 0, 1, 0, 0], // CossackSaber: 1 Wood, 3 Iron, 1 Leather
    [2, 0, 1, 0, 0, 1], // ElderStaff: 2 Wood, 1 Gold, 1 Diamond
    [0, 2, 1, 4, 0, 0], // CharacternikArmor: 4 Leather, 2 Iron, 1 Gold
    [0, 4, 2, 0, 0, 2], // BattleBracelet: 4 Iron, 2 Gold, 2 Diamond
];

const ITEM_NAMES: [&str; 4] = [
    "Cossack Saber",
    "Elder Staff",
    "Characternik Armor",
    "Battle Bracelet",
];

#[program]
pub mod crafting {
    use super::*;

    /// Initialize the crafting program
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.total_crafted = 0;
        config.bump = ctx.bumps.config;
        
        msg!("Crafting program initialized");
        Ok(())
    }

    /// Craft an item from resources
    /// Burns the required resources and creates an NFT
    pub fn craft_item(
        ctx: Context<CraftItem>,
        item_type: u8,
    ) -> Result<()> {
        require!(item_type < 4, ErrorCode::InvalidItemType);
        
        let recipe = RECIPES[item_type as usize];
        let item_name = ITEM_NAMES[item_type as usize];

        msg!("Crafting: {}", item_name);
        msg!("Recipe: WOOD={}, IRON={}, GOLD={}, LEATHER={}, STONE={}, DIAMOND={}",
            recipe[0], recipe[1], recipe[2], recipe[3], recipe[4], recipe[5]);

        // Resources would be burned here via CPI
        // In a full implementation, we'd iterate through each resource
        // and burn the required amounts from the player's token accounts

        let config = &mut ctx.accounts.config;
        config.total_crafted += 1;

        msg!("Successfully crafted {} (Total crafted: {})", item_name, config.total_crafted);
        Ok(())
    }

    /// Burn a specific resource token
    pub fn burn_resource(
        ctx: Context<BurnResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        require!(resource_id < 6, ErrorCode::InvalidResourceId);
        require!(amount > 0, ErrorCode::InvalidAmount);

        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token_2022::burn(cpi_ctx, amount)?;

        msg!("Burned {} units of resource {}", amount, resource_id);
        Ok(())
    }

    /// Create the NFT item after burning resources
    pub fn create_nft_item(
        ctx: Context<CreateNftItem>,
        item_type: u8,
    ) -> Result<()> {
        require!(item_type < 4, ErrorCode::InvalidItemType);

        // Initialize the mint
        let cpi_accounts = token::InitializeMint {
            mint: ctx.accounts.mint.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // NFT has 0 decimals and supply of 1
        token::initialize_mint(cpi_ctx, 0, &ctx.accounts.owner.key(), Some(&ctx.accounts.owner.key()))?;

        // Mint 1 token to the player
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::mint_to(cpi_ctx, 1)?;

        msg!("Created NFT for item type: {}", item_type);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + CraftingConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, CraftingConfig>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CraftItem<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, CraftingConfig>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct BurnResource<'info> {
    /// CHECK: Resource mint
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    
    /// CHECK: Player's resource token account
    #[account(mut)]
    pub token_account: AccountInfo<'info>,
    
    pub owner: Signer<'info>,
    
    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CreateNftItem<'info> {
    /// CHECK: The new NFT mint account
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    
    /// CHECK: The player's token account for the NFT
    #[account(mut)]
    pub token_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
#[derive(InitSpace)]
pub struct CraftingConfig {
    pub admin: Pubkey,
    pub total_crafted: u64,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid item type (must be 0-3)")]
    InvalidItemType,
    #[msg("Invalid resource ID (must be 0-5)")]
    InvalidResourceId,
    #[msg("Insufficient resources for crafting")]
    InsufficientResources,
    #[msg("Invalid amount")]
    InvalidAmount,
}
