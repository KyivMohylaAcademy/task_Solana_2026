use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Burn};
use anchor_spl::token_2022::{self, Token2022, MintTo};

declare_id!("5pZRWZiRXhd6NJ67v1B4TzAGDrVzrH6cFysU8KME3rdE");

#[program]
pub mod marketplace {
    use super::*;

    /// Initialize the marketplace
    pub fn initialize(ctx: Context<Initialize>, item_prices: [u64; 4]) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.item_prices = item_prices;
        config.total_sales = 0;
        config.bump = ctx.bumps.config;
        
        msg!("Marketplace initialized with prices: {:?}", item_prices);
        Ok(())
    }

    /// List an item for sale
    pub fn list_item(
        ctx: Context<ListItem>,
        item_type: u8,
        custom_price: Option<u64>,
    ) -> Result<()> {
        require!(item_type < 4, ErrorCode::InvalidItemType);

        let listing = &mut ctx.accounts.listing;
        listing.seller = ctx.accounts.seller.key();
        listing.item_mint = ctx.accounts.item_mint.key();
        listing.item_type = item_type;
        listing.price = custom_price.unwrap_or(ctx.accounts.config.item_prices[item_type as usize]);
        listing.is_active = true;
        listing.bump = ctx.bumps.listing;

        msg!("Item listed for sale: type={}, price={}", item_type, listing.price);
        Ok(())
    }

    /// Buy an item (burns NFT, mints MagicToken to seller)
    pub fn buy_item(ctx: Context<BuyItem>) -> Result<()> {
        let listing = &ctx.accounts.listing;
        require!(listing.is_active, ErrorCode::ListingNotActive);

        let price = listing.price;
        let item_type = listing.item_type;

        msg!("Processing purchase: item_type={}, price={}", item_type, price);

        // Burn the NFT
        let cpi_accounts = Burn {
            mint: ctx.accounts.item_mint.to_account_info(),
            from: ctx.accounts.item_token_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, 1)?;

        msg!("NFT burned");

        // Mint MagicToken to seller via CPI
        let config_seeds: &[&[u8]] = &[
            b"marketplace_authority",
            &[ctx.accounts.config.bump],
        ];
        let signer = &[&config_seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.magic_token_mint.to_account_info(),
            to: ctx.accounts.seller_magic_token_account.to_account_info(),
            authority: ctx.accounts.marketplace_authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_2022_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token_2022::mint_to(cpi_ctx, price)?;

        msg!("Minted {} MagicTokens to seller", price);

        // Update statistics
        let config = &mut ctx.accounts.config;
        config.total_sales += 1;

        // Deactivate listing
        let listing = &mut ctx.accounts.listing;
        listing.is_active = false;

        msg!("Sale completed. Total sales: {}", config.total_sales);
        Ok(())
    }

    /// Cancel a listing
    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        let listing = &mut ctx.accounts.listing;
        require!(listing.is_active, ErrorCode::ListingNotActive);
        require!(listing.seller == ctx.accounts.seller.key(), ErrorCode::Unauthorized);

        listing.is_active = false;

        msg!("Listing cancelled for item: {}", listing.item_mint);
        Ok(())
    }

    /// Update item prices (admin only)
    pub fn update_prices(
        ctx: Context<UpdatePrices>,
        new_prices: [u64; 4],
    ) -> Result<()> {
        require!(
            ctx.accounts.config.admin == ctx.accounts.admin.key(),
            ErrorCode::Unauthorized
        );

        ctx.accounts.config.item_prices = new_prices;

        msg!("Prices updated: {:?}", new_prices);
        Ok(())
    }

    /// Sell item directly (combined list + buy in one transaction)
    pub fn sell_item_direct(
        ctx: Context<SellItemDirect>,
        item_type: u8,
    ) -> Result<()> {
        require!(item_type < 4, ErrorCode::InvalidItemType);

        let price = ctx.accounts.config.item_prices[item_type as usize];

        msg!("Direct sale: item_type={}, price={}", item_type, price);

        // Burn the NFT
        let cpi_accounts = Burn {
            mint: ctx.accounts.item_mint.to_account_info(),
            from: ctx.accounts.item_token_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, 1)?;

        msg!("NFT burned");

        // Mint MagicToken to seller
        let config_seeds: &[&[u8]] = &[
            b"marketplace_authority",
            &[ctx.accounts.config.bump],
        ];
        let signer = &[&config_seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.magic_token_mint.to_account_info(),
            to: ctx.accounts.seller_magic_token_account.to_account_info(),
            authority: ctx.accounts.marketplace_authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_2022_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token_2022::mint_to(cpi_ctx, price)?;

        msg!("Minted {} MagicTokens to seller", price);

        // Update statistics
        let config = &mut ctx.accounts.config;
        config.total_sales += 1;

        msg!("Direct sale completed. Total sales: {}", config.total_sales);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + MarketplaceConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, MarketplaceConfig>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct ListItem<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, MarketplaceConfig>,
    
    #[account(
        init,
        payer = seller,
        space = 8 + ItemListing::INIT_SPACE,
        seeds = [b"listing", item_mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, ItemListing>,
    
    /// CHECK: The NFT mint being listed
    pub item_mint: AccountInfo<'info>,
    
    #[account(mut)]
    pub seller: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyItem<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, MarketplaceConfig>,
    
    #[account(
        mut,
        seeds = [b"listing", item_mint.key().as_ref()],
        bump = listing.bump,
        close = seller
    )]
    pub listing: Account<'info, ItemListing>,
    
    /// CHECK: PDA authority for marketplace
    #[account(
        seeds = [b"marketplace_authority"],
        bump
    )]
    pub marketplace_authority: AccountInfo<'info>,
    
    /// CHECK: The NFT mint being sold
    #[account(mut)]
    pub item_mint: AccountInfo<'info>,
    
    /// CHECK: Seller's NFT token account
    #[account(mut)]
    pub item_token_account: AccountInfo<'info>,
    
    /// CHECK: MagicToken mint
    #[account(mut)]
    pub magic_token_mint: AccountInfo<'info>,
    
    /// CHECK: Seller's MagicToken account
    #[account(mut)]
    pub seller_magic_token_account: AccountInfo<'info>,
    
    /// CHECK: The seller
    #[account(mut)]
    pub seller: AccountInfo<'info>,
    
    pub buyer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(
        mut,
        seeds = [b"listing", item_mint.key().as_ref()],
        bump = listing.bump
    )]
    pub listing: Account<'info, ItemListing>,
    
    /// CHECK: The NFT mint
    pub item_mint: AccountInfo<'info>,
    
    pub seller: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdatePrices<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, MarketplaceConfig>,
    
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct SellItemDirect<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, MarketplaceConfig>,
    
    /// CHECK: PDA authority for marketplace
    #[account(
        seeds = [b"marketplace_authority"],
        bump
    )]
    pub marketplace_authority: AccountInfo<'info>,
    
    /// CHECK: The NFT mint being sold
    #[account(mut)]
    pub item_mint: AccountInfo<'info>,
    
    /// CHECK: Seller's NFT token account
    #[account(mut)]
    pub item_token_account: AccountInfo<'info>,
    
    /// CHECK: MagicToken mint
    #[account(mut)]
    pub magic_token_mint: AccountInfo<'info>,
    
    /// CHECK: Seller's MagicToken account
    #[account(mut)]
    pub seller_magic_token_account: AccountInfo<'info>,
    
    #[account(mut)]
    pub seller: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
}

#[account]
#[derive(InitSpace)]
pub struct MarketplaceConfig {
    pub admin: Pubkey,
    pub item_prices: [u64; 4],
    pub total_sales: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ItemListing {
    pub seller: Pubkey,
    pub item_mint: Pubkey,
    pub item_type: u8,
    pub price: u64,
    pub is_active: bool,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid item type (must be 0-3)")]
    InvalidItemType,
    #[msg("Listing is not active")]
    ListingNotActive,
    #[msg("Unauthorized")]
    Unauthorized,
}
