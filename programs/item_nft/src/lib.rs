//! # item_nft
//!
//! Manages crafted-item NFTs using **Metaplex Core** (`mpl-core`).
//! Each item is a Core asset belonging to a shared collection.
//! An `ItemMetadata` PDA tracks `item_type`, `owner`, and `mint` (the asset address).
//!
//! ## Authority model
//! - `mint_item`  requires a `crafting_authority` PDA signer from the crafting program.
//! - `burn_item`  requires a `marketplace_authority` PDA signer from the marketplace program.
//! - Collection update authority = `item_collection_authority` PDA of this program.

use anchor_lang::prelude::*;
use mpl_core::{
    ID as MPL_CORE_ID,
    instructions::{
        CreateCollectionV2CpiBuilder,
        CreateV2CpiBuilder,
        BurnV1CpiBuilder,
    },
    types::{
        Attribute, Attributes, Plugin, PluginAuthority,
        PluginAuthorityPair,
    },
};
use shared::{seeds::*, state::{ItemMetadata, ItemNftConfig}, errors::GameError, items::ItemKind};

declare_id!("FQ4ptApSkc8RjUW35BVqL8BeuMgMRSYGtzDEwy2GhERf");

pub mod crafting_program {
    anchor_lang::declare_id!("B2mXTz3cVrn3UubqVTKyqyEWh6qTiVcCjn1DQw8azB65");
}

pub mod marketplace_program {
    anchor_lang::declare_id!("8FCw32yjvmK8po3yjH3U6p4ZNSzm7H7iCWiwjR6JHkzx");
}

#[program]
pub mod item_nft {
    use super::*;

    /// Create the mpl-core collection that all crafted items belong to. Admin only.
    pub fn initialize_collection(ctx: Context<InitializeCollection>) -> Result<()> {
        let authority_bump = ctx.bumps.collection_authority;
        let auth_seeds: &[&[u8]] = &[ITEM_COLLECTION_AUTHORITY_SEED, &[authority_bump]];
        let signer_seeds = &[auth_seeds];

        CreateCollectionV2CpiBuilder::new(&ctx.accounts.mpl_core_program)
            .collection(&ctx.accounts.collection)
            .update_authority(Some(&ctx.accounts.collection_authority))
            .payer(&ctx.accounts.admin)
            .name("Козацький бізнес — Предмети".to_string())
            .uri("https://kozatskyi-biznes.example/collection.json".to_string())
            .system_program(&ctx.accounts.system_program)
            .invoke_signed(signer_seeds)?;

        let config = &mut ctx.accounts.item_nft_config;
        config.admin = ctx.accounts.admin.key();
        config.collection = ctx.accounts.collection.key();
        config.bump = ctx.bumps.item_nft_config;

        Ok(())
    }

    /// Mint a new item NFT to `recipient`. Only callable via CPI from the crafting program.
    /// The `asset` account must be a new keypair whose address is also passed as a signer
    /// so mpl-core can initialize it.
    pub fn mint_item(
        ctx: Context<MintItem>,
        item_type: u8,
        recipient: Pubkey,
    ) -> Result<()> {
        let kind = ItemKind::from_u8(item_type).ok_or(GameError::InvalidItemType)?;

        let authority_bump = ctx.bumps.collection_authority;
        let auth_seeds: &[&[u8]] = &[ITEM_COLLECTION_AUTHORITY_SEED, &[authority_bump]];
        let signer_seeds = &[auth_seeds];

        // Build attributes plugin embedding the item_type on-chain
        let attributes_plugin = PluginAuthorityPair {
            plugin: Plugin::Attributes(Attributes {
                attribute_list: vec![
                    Attribute {
                        key: "item_type".to_string(),
                        value: item_type.to_string(),
                    },
                    Attribute {
                        key: "item_name".to_string(),
                        value: kind.name().to_string(),
                    },
                ],
            }),
            authority: Some(PluginAuthority::UpdateAuthority),
        };

        CreateV2CpiBuilder::new(&ctx.accounts.mpl_core_program)
            .asset(&ctx.accounts.asset)
            .collection(Some(&ctx.accounts.collection))
            .authority(Some(&ctx.accounts.collection_authority))
            .payer(&ctx.accounts.payer)
            .owner(Some(&ctx.accounts.recipient))
            .system_program(&ctx.accounts.system_program)
            .name(kind.name().to_string())
            .uri(format!(
                "https://kozatskyi-biznes.example/items/{}.json",
                item_type
            ))
            .plugins(vec![attributes_plugin])
            .invoke_signed(signer_seeds)?;

        // Create ItemMetadata PDA
        let metadata = &mut ctx.accounts.item_metadata;
        metadata.item_type = item_type;
        metadata.owner = recipient;
        metadata.mint = ctx.accounts.asset.key();
        metadata.bump = ctx.bumps.item_metadata;

        Ok(())
    }

    /// Burn an item NFT. Only callable via CPI from the marketplace program.
    /// Closes the ItemMetadata PDA and refunds rent to the original owner.
    pub fn burn_item(ctx: Context<BurnItem>) -> Result<()> {
        let authority_bump = ctx.bumps.collection_authority;
        let auth_seeds: &[&[u8]] = &[ITEM_COLLECTION_AUTHORITY_SEED, &[authority_bump]];
        let signer_seeds = &[auth_seeds];

        BurnV1CpiBuilder::new(&ctx.accounts.mpl_core_program)
            .asset(&ctx.accounts.asset)
            .collection(Some(&ctx.accounts.collection))
            .payer(&ctx.accounts.payer)
            .authority(Some(&ctx.accounts.collection_authority))
            .system_program(Some(&ctx.accounts.system_program))
            .invoke_signed(signer_seeds)?;

        // item_metadata PDA will be closed by Anchor (close = payer constraint)
        Ok(())
    }
}

// ─── Account structs ──────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct InitializeCollection<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// New mpl-core collection asset – must be a fresh keypair passed as signer.
    /// CHECK: Initialized by mpl-core.
    #[account(mut)]
    pub collection: Signer<'info>,

    /// PDA used as the collection's update authority.
    /// CHECK: Verified by seeds.
    #[account(
        seeds = [ITEM_COLLECTION_AUTHORITY_SEED],
        bump,
    )]
    pub collection_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin,
        space = ItemNftConfig::LEN,
        seeds = [b"item_nft_config"],
        bump,
    )]
    pub item_nft_config: Account<'info, ItemNftConfig>,

    /// CHECK: mpl-core program.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(item_type: u8, recipient: Pubkey)]
pub struct MintItem<'info> {
    /// crafting_authority PDA from the crafting program.
    #[account(
        seeds = [CRAFTING_AUTHORITY_SEED],
        bump,
        seeds::program = crafting_program::ID,
    )]
    pub crafting_authority: Signer<'info>,

    /// New mpl-core asset keypair – must be generated off-chain and passed as signer.
    /// CHECK: Initialized by mpl-core.
    #[account(mut)]
    pub asset: Signer<'info>,

    /// The collection this asset belongs to.
    /// CHECK: Verified against item_nft_config.
    #[account(
        mut,
        constraint = item_nft_config.collection == collection.key() @ GameError::UnauthorizedCaller,
    )]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: Collection update authority PDA.
    #[account(
        seeds = [ITEM_COLLECTION_AUTHORITY_SEED],
        bump,
    )]
    pub collection_authority: UncheckedAccount<'info>,

    /// The wallet to receive the NFT.
    /// CHECK: Passed to mpl-core as owner.
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    /// Pays rent for the asset account.
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = ItemMetadata::LEN,
        seeds = [ITEM_SEED, asset.key().as_ref()],
        bump,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    #[account(
        seeds = [b"item_nft_config"],
        bump = item_nft_config.bump,
    )]
    pub item_nft_config: Account<'info, ItemNftConfig>,

    /// CHECK: mpl-core program.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BurnItem<'info> {
    /// marketplace_authority PDA from the marketplace program.
    #[account(
        seeds = [MARKETPLACE_AUTHORITY_SEED],
        bump,
        seeds::program = marketplace_program::ID,
    )]
    pub marketplace_authority: Signer<'info>,

    /// The mpl-core asset to burn.
    /// CHECK: Validated by mpl-core.
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// CHECK: Validated against config.
    #[account(
        mut,
        constraint = item_nft_config.collection == collection.key() @ GameError::UnauthorizedCaller,
    )]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: Collection update authority PDA.
    #[account(
        seeds = [ITEM_COLLECTION_AUTHORITY_SEED],
        bump,
    )]
    pub collection_authority: UncheckedAccount<'info>,

    /// Receives the rent refund from the closed item_metadata PDA.
    /// CHECK: Validated to be the original owner.
    #[account(
        mut,
        constraint = item_metadata.owner == payer.key() @ GameError::WrongOwner,
    )]
    pub payer: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [ITEM_SEED, asset.key().as_ref()],
        bump = item_metadata.bump,
        close = payer,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    #[account(
        seeds = [b"item_nft_config"],
        bump = item_nft_config.bump,
    )]
    pub item_nft_config: Account<'info, ItemNftConfig>,

    /// CHECK: mpl-core program.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
