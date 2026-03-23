use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

pub mod errors;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("ELwieeZdc4BtAy4Fc8YzKEZsw86YaxZNHRnPdrMjHt7q");

#[program]
pub mod marketplace {
    use super::*;

    /// Sell an item directly to the game at a fixed price.
    /// Burns the NFT via item_nft and mints MagicToken to the seller.
    /// remaining_accounts: [metadata_account, master_edition, metadata_program, sysvar_instructions]
    pub fn sell_item<'info>(
        ctx: Context<'_, '_, 'info, 'info, SellItem<'info>>,
    ) -> Result<()> {
        let item_type = ctx.accounts.item_metadata.item_type;
        require!(
            (item_type as usize) < ctx.accounts.game_config.item_prices.len(),
            MarketError::InvalidItemType
        );
        let price = ctx.accounts.game_config.item_prices[item_type as usize];

        let caller_bump = ctx.bumps.caller_authority;
        let caller_seeds: &[&[u8]] = &[b"caller_authority", &[caller_bump]];

        let remaining = ctx.remaining_accounts;
        require!(remaining.len() >= 4, MarketError::InvalidRemainingAccounts);

        let metadata_account = remaining[0].to_account_info();
        let master_edition = remaining[1].to_account_info();
        let metadata_program = remaining[2].to_account_info();
        let sysvar_instructions = remaining[3].to_account_info();

        item_nft::cpi::burn_item(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                item_nft::cpi::accounts::BurnItem {
                    caller_authority: ctx.accounts.caller_authority.to_account_info(),
                    config: ctx.accounts.item_nft_config.to_account_info(),
                    player: ctx.accounts.seller.to_account_info(),
                    item_metadata: ctx.accounts.item_metadata.to_account_info(),
                    nft_mint: ctx.accounts.nft_mint.to_account_info(),
                    player_nft_ata: ctx.accounts.seller_nft_ata.to_account_info(),
                    metadata_account,
                    master_edition,
                    metadata_program,
                    sysvar_instructions,
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
                &[caller_seeds],
            ),
        )?;

        magic_token::cpi::mint_magic_token(
            CpiContext::new_with_signer(
                ctx.accounts.magic_token_program.to_account_info(),
                magic_token::cpi::accounts::MintMagicToken {
                    caller_authority: ctx.accounts.caller_authority.to_account_info(),
                    config: ctx.accounts.magic_token_config.to_account_info(),
                    mint: ctx.accounts.magic_token_mint.to_account_info(),
                    mint_authority: ctx.accounts.magic_mint_authority.to_account_info(),
                    recipient_ata: ctx.accounts.seller_magic_ata.to_account_info(),
                    token_program: ctx.accounts.token_2022_program.to_account_info(),
                },
                &[caller_seeds],
            ),
            price,
        )?;

        Ok(())
    }

    /// List an NFT for sale on the marketplace. Transfers NFT to escrow.
    pub fn list_item(ctx: Context<ListItem>, price: u64) -> Result<()> {
        require!(price > 0, MarketError::InvalidPrice);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.seller_nft_ata.to_account_info(),
                    to: ctx.accounts.escrow_nft_ata.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            1,
        )?;

        let listing = &mut ctx.accounts.listing;
        listing.seller = ctx.accounts.seller.key();
        listing.item_mint = ctx.accounts.nft_mint.key();
        listing.item_type = ctx.accounts.item_metadata.item_type;
        listing.price = price;
        listing.bump = ctx.bumps.listing;
        listing.escrow_bump = ctx.bumps.escrow_authority;

        Ok(())
    }

    /// Buy a listed item. Transfers MagicToken from buyer to seller,
    /// and NFT from escrow to buyer.
    pub fn buy_item(ctx: Context<BuyItem>) -> Result<()> {
        let price = ctx.accounts.listing.price;
        let item_mint_key = ctx.accounts.listing.item_mint;
        let escrow_bump = ctx.accounts.listing.escrow_bump;

        anchor_spl::token_2022::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_2022_program.to_account_info(),
                anchor_spl::token_2022::TransferChecked {
                    from: ctx.accounts.buyer_magic_ata.to_account_info(),
                    mint: ctx.accounts.magic_token_mint.to_account_info(),
                    to: ctx.accounts.seller_magic_ata.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            price,
            0,
        )?;

        let escrow_seeds: &[&[u8]] = &[b"escrow", item_mint_key.as_ref(), &[escrow_bump]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_nft_ata.to_account_info(),
                    to: ctx.accounts.buyer_nft_ata.to_account_info(),
                    authority: ctx.accounts.escrow_authority.to_account_info(),
                },
                &[escrow_seeds],
            ),
            1,
        )?;

        let caller_bump = ctx.bumps.caller_authority;
        let caller_seeds: &[&[u8]] = &[b"caller_authority", &[caller_bump]];

        item_nft::cpi::transfer_item_ownership(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                item_nft::cpi::accounts::TransferItemOwnership {
                    caller_authority: ctx.accounts.caller_authority.to_account_info(),
                    config: ctx.accounts.item_nft_config.to_account_info(),
                    item_metadata: ctx.accounts.item_metadata.to_account_info(),
                },
                &[caller_seeds],
            ),
            ctx.accounts.buyer.key(),
        )?;

        Ok(())
    }

    /// Cancel a listing and return the NFT to the seller.
    pub fn delist_item(ctx: Context<DelistItem>) -> Result<()> {
        let item_mint_key = ctx.accounts.listing.item_mint;
        let escrow_bump = ctx.accounts.listing.escrow_bump;

        let escrow_seeds: &[&[u8]] = &[b"escrow", item_mint_key.as_ref(), &[escrow_bump]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_nft_ata.to_account_info(),
                    to: ctx.accounts.seller_nft_ata.to_account_info(),
                    authority: ctx.accounts.escrow_authority.to_account_info(),
                },
                &[escrow_seeds],
            ),
            1,
        )?;

        Ok(())
    }
}
