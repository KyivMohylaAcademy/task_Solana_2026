//! Marketplace: escrow NFTs on list, settle MagicToken on purchase, burn NFT from escrow.

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::{self, Burn, Token2022, TransferChecked};
use anchor_spl::token_interface::{Mint, TokenAccount};
use magic_token::cpi::{
    accounts::BurnFromBuyer, accounts::MintToSeller, burn_from_buyer, mint_to_seller,
};

declare_id!("CCGbViKpgMcJL7zjh6C3pSsWbHrHpJoxaB7yAJVTkqPa");

#[account]
pub struct Listing {
    pub seller: Pubkey,
    pub item_mint: Pubkey,
    pub price: u64,
    pub bump: u8,
}

impl Listing {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 1;
}

pub const MARKET_EXEC_SEED: &[u8] = magic_token::MARKET_EXEC_SEED;

#[program]
pub mod marketplace {
    use super::*;

    /// Lists an item: records price and moves the NFT into a marketplace-owned escrow ATA.
    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        require!(price > 0, MarketplaceError::InvalidPrice);
        require_keys_eq!(
            ctx.accounts.seller.key(),
            ctx.accounts.seller_item_ata.owner,
            MarketplaceError::NotItemOwner
        );
        require_keys_eq!(
            ctx.accounts.seller_item_ata.mint,
            ctx.accounts.item_mint.key(),
            MarketplaceError::MintMismatch
        );

        token_2022::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    mint: ctx.accounts.item_mint.to_account_info(),
                    from: ctx.accounts.seller_item_ata.to_account_info(),
                    to: ctx.accounts.escrow_item_ata.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            1,
            ctx.accounts.item_mint.decimals,
        )?;

        let listing = &mut ctx.accounts.listing;
        listing.seller = ctx.accounts.seller.key();
        listing.item_mint = ctx.accounts.item_mint.key();
        listing.price = price;
        listing.bump = ctx.bumps.listing;

        Ok(())
    }

    /// Removes a listing and returns the NFT from escrow to the seller.
    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.listing.seller,
            ctx.accounts.seller.key(),
            MarketplaceError::NotListingOwner
        );

        let bump = ctx.bumps.market_authority;
        let seeds: &[&[u8]] = &[MARKET_EXEC_SEED, &[bump]];
        let signer = &[seeds];

        token_2022::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    mint: ctx.accounts.item_mint.to_account_info(),
                    from: ctx.accounts.escrow_item_ata.to_account_info(),
                    to: ctx.accounts.seller_item_ata.to_account_info(),
                    authority: ctx.accounts.market_authority.to_account_info(),
                },
                signer,
            ),
            1,
            ctx.accounts.item_mint.decimals,
        )?;

        Ok(())
    }

    /// Purchases a listed item: burns MagicToken from buyer, mints to seller, burns NFT from escrow.
    pub fn purchase(ctx: Context<Purchase>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.listing.seller,
            ctx.accounts.seller.key(),
            MarketplaceError::SellerMismatch
        );
        require!(
            ctx.accounts.buyer.key() != ctx.accounts.seller.key(),
            MarketplaceError::SellerCannotBuy
        );

        let price = ctx.accounts.listing.price;

        let market_bump = ctx.bumps.market_authority;
        let market_seeds: &[&[u8]] = &[MARKET_EXEC_SEED, &[market_bump]];
        let market_signer = &[market_seeds];

        let burn_accounts = BurnFromBuyer {
            config: ctx.accounts.config.to_account_info(),
            marketplace_authority: ctx.accounts.market_authority.to_account_info(),
            mint: ctx.accounts.magic_mint.to_account_info(),
            buyer_ata: ctx.accounts.buyer_magic_ata.to_account_info(),
            buyer: ctx.accounts.buyer.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            marketplace_program: ctx.accounts.marketplace_program.to_account_info(),
        };

        burn_from_buyer(
            CpiContext::new_with_signer(
                ctx.accounts.magic_token_program.to_account_info(),
                burn_accounts,
                market_signer,
            ),
            price,
        )?;

        let mint_accounts = MintToSeller {
            config: ctx.accounts.config.to_account_info(),
            marketplace_authority: ctx.accounts.market_authority.to_account_info(),
            mint: ctx.accounts.magic_mint.to_account_info(),
            seller_ata: ctx.accounts.seller_magic_ata.to_account_info(),
            mint_authority: ctx.accounts.magic_mint_authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            marketplace_program: ctx.accounts.marketplace_program.to_account_info(),
        };

        mint_to_seller(
            CpiContext::new_with_signer(
                ctx.accounts.magic_token_program.to_account_info(),
                mint_accounts,
                market_signer,
            ),
            price,
        )?;

        token_2022::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.item_mint.to_account_info(),
                    from: ctx.accounts.escrow_item_ata.to_account_info(),
                    authority: ctx.accounts.market_authority.to_account_info(),
                },
                market_signer,
            ),
            1,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct List<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    pub item_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub seller_item_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = seller,
        associated_token::mint = item_mint,
        associated_token::authority = market_authority,
        associated_token::token_program = token_program,
    )]
    pub escrow_item_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = seller,
        space = Listing::LEN,
        seeds = [b"listing", item_mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,
    /// CHECK: Marketplace executor PDA (`MARKET_EXEC_SEED`); signs escrow transfers.
    #[account(seeds = [MARKET_EXEC_SEED], bump)]
    pub market_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Delist<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    pub item_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub seller_item_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_item_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"listing", item_mint.key().as_ref()],
        bump = listing.bump,
        close = seller,
        constraint = listing.seller == seller.key() @ MarketplaceError::NotListingOwner,
        constraint = listing.item_mint == item_mint.key() @ MarketplaceError::MintMismatch
    )]
    pub listing: Account<'info, Listing>,
    /// CHECK: Marketplace executor PDA; signs NFT return from escrow.
    #[account(mut, seeds = [MARKET_EXEC_SEED], bump)]
    pub market_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct Purchase<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: must match listing.seller
    pub seller: UncheckedAccount<'info>,
    pub item_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub escrow_item_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub buyer_magic_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub seller_magic_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"listing", item_mint.key().as_ref()],
        bump = listing.bump,
        close = buyer,
        constraint = listing.seller == seller.key() @ MarketplaceError::SellerMismatch,
        constraint = listing.item_mint == item_mint.key() @ MarketplaceError::MintMismatch
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        seeds = [b"magic_config"],
        bump = config.bump,
        seeds::program = magic_token::ID
    )]
    pub config: Account<'info, magic_token::MagicTokenConfig>,
    /// CHECK: Magic token mint authority PDA; constrained by seeds.
    #[account(mut, seeds = [b"magic_auth"], bump)]
    pub magic_mint_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub magic_mint: InterfaceAccount<'info, Mint>,
    /// CHECK: Marketplace executor PDA; signs magic_token CPIs and NFT burn from escrow.
    #[account(mut, seeds = [MARKET_EXEC_SEED], bump)]
    pub market_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    /// CHECK: magic_token program
    pub magic_token_program: UncheckedAccount<'info>,
    /// CHECK: must match config.marketplace_program
    pub marketplace_program: UncheckedAccount<'info>,
}

#[error_code]
pub enum MarketplaceError {
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Not item owner")]
    NotItemOwner,
    #[msg("Not listing owner")]
    NotListingOwner,
    #[msg("Seller mismatch")]
    SellerMismatch,
    #[msg("Seller cannot buy own listing")]
    SellerCannotBuy,
    #[msg("Mint mismatch")]
    MintMismatch,
}
