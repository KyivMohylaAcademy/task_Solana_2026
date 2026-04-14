//! # crafting
//!
//! Handles item crafting: burns the required resources (via CPI to `resource_manager`)
//! and mints the resulting NFT (via CPI to `item_nft`).
//!
//! Recipes (resource amounts per item type):
//! ```text
//! 0 – Cossack Saber:      1 Wood + 3 Iron + 1 Leather
//! 1 – Elder's Staff:      2 Wood + 1 Gold  + 1 Diamond
//! 2 – Kharakternyk Armor: 4 Leather + 2 Iron + 1 Gold
//! 3 – Battle Bracelet:    4 Iron + 2 Gold + 2 Diamond
//! ```
//!
//! ## Authority model
//! This program signs CPI calls using its `crafting_authority` PDA, which is verified
//! by `resource_manager` and `item_nft` via `seeds::program = crafting::ID`.

use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use resource_manager::{
    self,
    cpi::{
        accounts::{BurnFromCrafting as RmBurnFromCrafting, MintFromCrafting as RmMintFromCrafting},
        burn_from_crafting, mint_from_crafting,
    },
    program::ResourceManager,
};
use item_nft::{
    self,
    cpi::{accounts::MintItem as NftMintItem, mint_item},
    program::ItemNft,
};
use shared::{
    seeds::*,
    state::GameConfig,
    errors::GameError,
    items::{ItemKind, RECIPES},
};
use mpl_core::ID as MPL_CORE_ID;

declare_id!("B2mXTz3cVrn3UubqVTKyqyEWh6qTiVcCjn1DQw8azB65");

#[program]
pub mod crafting {
    use super::*;

    /// Craft an item NFT by burning the required resources.
    ///
    /// `remaining_accounts` layout (in order of RECIPES resource indexes 0..5,
    /// only non-zero recipe entries are needed but all 6 pairs must be present;
    /// pass the same account for unused resources):
    ///   [mint_0, ata_0, mint_1, ata_1, ..., mint_5, ata_5]
    /// Then after the 12 resource accounts:
    ///   [asset_signer, recipient, item_metadata, item_nft_config, collection, payer]
    pub fn craft_item(ctx: Context<CraftItem>, item_type: u8) -> Result<()> {
        let kind = ItemKind::from_u8(item_type).ok_or(GameError::InvalidItemType)?;
        let recipe = RECIPES[item_type as usize];

        let crafting_bump = ctx.bumps.crafting_authority;
        let craft_seeds: &[&[u8]] = &[CRAFTING_AUTHORITY_SEED, &[crafting_bump]];
        let signer_seeds = &[craft_seeds];

        let rm_program = ctx.accounts.resource_manager_program.to_account_info();
        let rm_resource_authority = ctx.accounts.resource_authority.to_account_info();

        let remaining = ctx.remaining_accounts;
        // First 12 accounts: 6 pairs of (mint, ata)
        require!(remaining.len() >= 12, GameError::Overflow);

        // ── Burn resources ────────────────────────────────────────────────────
        for res_kind in 0usize..6 {
            let amount = recipe[res_kind];
            if amount == 0 {
                continue;
            }
            let mint_info = &remaining[res_kind * 2];
            let ata_info = &remaining[res_kind * 2 + 1];

            burn_from_crafting(
                CpiContext::new_with_signer(
                    rm_program.clone(),
                    RmBurnFromCrafting {
                        crafting_authority: ctx.accounts.crafting_authority.to_account_info(),
                        player: ctx.accounts.player.to_account_info(),
                        mint: mint_info.clone(),
                        player_ata: ata_info.clone(),
                        token_program: ctx.accounts.token_program.to_account_info(),
                    },
                    signer_seeds,
                ),
                res_kind as u8,
                amount,
            )?;
        }

        // ── Mint NFT ──────────────────────────────────────────────────────────
        // remaining_accounts[12..]:
        //   [asset, recipient, item_metadata, item_nft_config, collection, collection_authority, payer]
        require!(remaining.len() >= 18, GameError::Overflow);
        let asset = &remaining[12];
        let recipient = &remaining[13];
        let item_metadata = &remaining[14];
        let item_nft_config = &remaining[15];
        let collection = &remaining[16];
        let collection_authority = &remaining[17];

        mint_item(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                NftMintItem {
                    crafting_authority: ctx.accounts.crafting_authority.to_account_info(),
                    asset: asset.clone(),
                    collection: collection.clone(),
                    collection_authority: collection_authority.clone(),
                    recipient: recipient.clone(),
                    payer: ctx.accounts.player.to_account_info(),
                    item_metadata: item_metadata.clone(),
                    item_nft_config: item_nft_config.clone(),
                    mpl_core_program: ctx.accounts.mpl_core_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
                signer_seeds,
            ),
            item_type,
            ctx.accounts.player.key(),
        )?;

        Ok(())
    }
}

// ─── Account structs ──────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct CraftItem<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    /// crafting_authority PDA – signed by this program for CPIs.
    /// CHECK: PDA verified by seeds.
    #[account(
        seeds = [CRAFTING_AUTHORITY_SEED],
        bump,
    )]
    pub crafting_authority: UncheckedAccount<'info>,

    /// resource_authority from resource_manager, passed through.
    /// CHECK: Verified inside resource_manager.
    #[account(
        seeds = [RESOURCE_AUTHORITY_SEED],
        bump,
        seeds::program = resource_manager::ID,
    )]
    pub resource_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID,
    )]
    pub game_config: Account<'info, GameConfig>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub item_nft_program: Program<'info, ItemNft>,

    /// CHECK: mpl-core program.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    // remaining_accounts: [mint_0,ata_0,...,mint_5,ata_5, asset,recipient,item_metadata,
    //                       item_nft_config,collection,collection_authority]
}
