//! # marketplace
//!
//! Players sell crafted items (NFTs) back to the game for MagicToken.
//! The NFT is burned and the seller receives `GameConfig.item_prices[item_type]` MagicTokens.
//!
//! ## Flow
//! 1. Verify `ItemMetadata.owner == seller` and `item_metadata.item_type == item_type`.
//! 2. CPI `item_nft::burn_item` (signed with `marketplace_authority`).
//! 3. CPI `magic_token::mint_to_player(price)` (signed with `marketplace_authority`).
//!
//! ## Authority model
//! This program signs CPIs using its `marketplace_authority` PDA, verified by
//! the callee programs via `seeds::program = marketplace::ID`.

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint as InterfaceMint, TokenAccount as InterfaceTokenAccount},
};
use item_nft::{
    self,
    cpi::{accounts::BurnItem as NftBurnItem, burn_item},
    program::ItemNft,
};
use magic_token::{
    self,
    cpi::{accounts::MintToPlayer as MtMintToPlayer, mint_to_player},
    program::MagicToken,
};
use resource_manager;
use shared::{
    seeds::*,
    state::{GameConfig, ItemMetadata},
    errors::GameError,
    items::ItemKind,
};
use mpl_core::ID as MPL_CORE_ID;

declare_id!("8FCw32yjvmK8po3yjH3U6p4ZNSzm7H7iCWiwjR6JHkzx");

#[program]
pub mod marketplace {
    use super::*;

    /// Sell a crafted item NFT for MagicToken.
    ///
    /// `remaining_accounts` layout:
    ///   [collection, collection_authority, item_nft_config,
    ///    magic_authority, magic_mint, seller_magic_ata]
    pub fn sell_item(ctx: Context<SellItem>, item_type: u8) -> Result<()> {
        // ── Validate ──────────────────────────────────────────────────────────
        ItemKind::from_u8(item_type).ok_or(GameError::InvalidItemType)?;

        let metadata = &ctx.accounts.item_metadata;
        require!(metadata.owner == ctx.accounts.seller.key(), GameError::WrongOwner);
        require!(metadata.item_type == item_type, GameError::ItemTypeMismatch);

        let price = ctx.accounts.game_config.item_prices[item_type as usize];

        let marketplace_bump = ctx.bumps.marketplace_authority;
        let mp_seeds: &[&[u8]] = &[MARKETPLACE_AUTHORITY_SEED, &[marketplace_bump]];
        let signer_seeds = &[mp_seeds];

        // ── CPI: burn NFT ─────────────────────────────────────────────────────
        let remaining = ctx.remaining_accounts;
        require!(remaining.len() >= 6, GameError::Overflow);

        let collection = &remaining[0];
        let collection_authority = &remaining[1];
        let item_nft_config = &remaining[2];
        let magic_authority = &remaining[3];
        let magic_mint = &remaining[4];
        let seller_magic_ata = &remaining[5];

        burn_item(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                NftBurnItem {
                    marketplace_authority: ctx.accounts.marketplace_authority.to_account_info(),
                    asset: ctx.accounts.asset.to_account_info(),
                    collection: collection.clone(),
                    collection_authority: collection_authority.clone(),
                    payer: ctx.accounts.seller.to_account_info(),
                    item_metadata: ctx.accounts.item_metadata.to_account_info(),
                    item_nft_config: item_nft_config.clone(),
                    mpl_core_program: ctx.accounts.mpl_core_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
                signer_seeds,
            ),
        )?;

        // ── CPI: mint MagicToken to seller ────────────────────────────────────
        mint_to_player(
            CpiContext::new_with_signer(
                ctx.accounts.magic_token_program.to_account_info(),
                MtMintToPlayer {
                    marketplace_authority: ctx.accounts.marketplace_authority.to_account_info(),
                    magic_authority: magic_authority.clone(),
                    magic_mint: magic_mint.clone(),
                    player_ata: seller_magic_ata.clone(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                signer_seeds,
            ),
            price,
        )?;

        Ok(())
    }
}

// ─── Account structs ──────────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct SellItem<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    /// marketplace_authority PDA – this program signs CPI calls with it.
    /// CHECK: PDA verified by seeds.
    #[account(
        seeds = [MARKETPLACE_AUTHORITY_SEED],
        bump,
    )]
    pub marketplace_authority: UncheckedAccount<'info>,

    /// The mpl-core asset to burn.
    /// CHECK: Validated by mpl-core / item_nft.
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// ItemMetadata PDA for the asset – validated inside burn_item CPI.
    #[account(
        mut,
        seeds = [ITEM_SEED, asset.key().as_ref()],
        bump = item_metadata.bump,
        seeds::program = item_nft::ID,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID,
    )]
    pub game_config: Account<'info, GameConfig>,

    pub item_nft_program: Program<'info, ItemNft>,
    pub magic_token_program: Program<'info, MagicToken>,

    /// CHECK: mpl-core program.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    // remaining_accounts:
    //   [collection, collection_authority, item_nft_config,
    //    magic_authority, magic_mint, seller_magic_ata]
}
