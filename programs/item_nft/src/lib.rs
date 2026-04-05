//! NFT minting and burning logic for craftable in-game items.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::ID as SYSVAR_INSTRUCTIONS_ID;
use anchor_spl::associated_token::{get_associated_token_address_with_program_id, AssociatedToken};
use anchor_spl::token::{Mint, Token, TokenAccount};
use mpl_token_metadata::accounts::{MasterEdition, Metadata};
use mpl_token_metadata::instructions::{BurnNftCpiBuilder, CreateV1CpiBuilder, MintV1CpiBuilder};
use mpl_token_metadata::types::{PrintSupply, TokenStandard};
use resource_manager::GameConfig;
use shared::{
    GameErrorCode, ItemType, CRAFTING_PROGRAM_ID, GAME_CONFIG_SEED, ITEM_COUNT, ITEM_METADATA_SEED,
    MARKETPLACE_PROGRAM_ID, PROGRAM_AUTHORITY_SEED, RESOURCE_MANAGER_PROGRAM_ID,
};

declare_id!("31YqF1ymwThcZTyGCmx6Uqnvjev15JRkWvMSJoxc3wve");

/// Owns item metadata accounts and forwards Metaplex mint and burn operations.
#[program]
pub mod item_nft {
    use super::*;

    /// Mints a one-of-one item NFT together with its metadata and master edition.
    pub fn mint_item_nft(
        ctx: Context<MintItemNft>,
        item_type: u8,
        uri: String,
        name: String,
        symbol: String,
    ) -> Result<()> {
        let item_type = parse_item_type(item_type)?;
        require!(
            ctx.accounts.caller_authority.is_signer,
            GameErrorCode::UnauthorizedItemMintCaller
        );

        assert_metaplex_accounts(
            &ctx.accounts.mint.key(),
            &ctx.accounts.metadata.key(),
            &ctx.accounts.master_edition.key(),
        )?;

        let expected_token_account = get_associated_token_address_with_program_id(
            &ctx.accounts.owner.key(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.token_program.key(),
        );
        require_keys_eq!(
            ctx.accounts.owner_item_token_account.key(),
            expected_token_account,
            GameErrorCode::InvalidItemTokenAccount
        );

        let authority_bump = ctx.bumps.program_authority;
        let signer_seeds: &[&[u8]] = &[PROGRAM_AUTHORITY_SEED, &[authority_bump]];

        CreateV1CpiBuilder::new(&ctx.accounts.token_metadata_program.to_account_info())
            .metadata(&ctx.accounts.metadata.to_account_info())
            .master_edition(Some(&ctx.accounts.master_edition.to_account_info()))
            .mint(&ctx.accounts.mint.to_account_info(), true)
            .authority(&ctx.accounts.program_authority.to_account_info())
            .payer(&ctx.accounts.owner.to_account_info())
            .update_authority(&ctx.accounts.program_authority.to_account_info(), true)
            .system_program(&ctx.accounts.system_program.to_account_info())
            .sysvar_instructions(&ctx.accounts.sysvar_instructions.to_account_info())
            .spl_token_program(Some(&ctx.accounts.token_program.to_account_info()))
            .name(name)
            .symbol(symbol)
            .uri(uri)
            .seller_fee_basis_points(0)
            .primary_sale_happened(false)
            .is_mutable(true)
            .token_standard(TokenStandard::NonFungible)
            .decimals(0)
            .print_supply(PrintSupply::Zero)
            .invoke_signed(&[signer_seeds])
            .map_err(anchor_lang::error::Error::from)?;

        MintV1CpiBuilder::new(&ctx.accounts.token_metadata_program.to_account_info())
            .token(&ctx.accounts.owner_item_token_account.to_account_info())
            .token_owner(Some(&ctx.accounts.owner.to_account_info()))
            .metadata(&ctx.accounts.metadata.to_account_info())
            .master_edition(Some(&ctx.accounts.master_edition.to_account_info()))
            .mint(&ctx.accounts.mint.to_account_info())
            .authority(&ctx.accounts.program_authority.to_account_info())
            .payer(&ctx.accounts.owner.to_account_info())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .sysvar_instructions(&ctx.accounts.sysvar_instructions.to_account_info())
            .spl_token_program(&ctx.accounts.token_program.to_account_info())
            .spl_ata_program(&ctx.accounts.associated_token_program.to_account_info())
            .amount(1)
            .invoke_signed(&[signer_seeds])
            .map_err(anchor_lang::error::Error::from)?;

        let item_metadata = &mut ctx.accounts.item_metadata;
        item_metadata.item_type = item_type as u8;
        item_metadata.owner = ctx.accounts.owner.key();
        item_metadata.mint = ctx.accounts.mint.key();
        item_metadata.bump = ctx.bumps.item_metadata;

        Ok(())
    }

    /// Burns an item NFT after validating the caller, item type and metadata PDAs.
    pub fn burn_item_nft(ctx: Context<BurnItemNft>, item_type: u8) -> Result<()> {
        let item_type = parse_item_type(item_type)?;
        require!(
            ctx.accounts.caller_authority.is_signer,
            GameErrorCode::UnauthorizedItemBurnCaller
        );

        assert_metaplex_accounts(
            &ctx.accounts.mint.key(),
            &ctx.accounts.metadata.key(),
            &ctx.accounts.master_edition.key(),
        )?;
        require_keys_eq!(
            ctx.accounts.item_metadata.mint,
            ctx.accounts.mint.key(),
            GameErrorCode::ItemMetadataMintMismatch
        );
        require_eq!(
            ctx.accounts.item_metadata.item_type,
            item_type as u8,
            GameErrorCode::ItemMetadataTypeMismatch
        );

        BurnNftCpiBuilder::new(&ctx.accounts.token_metadata_program.to_account_info())
            .metadata(&ctx.accounts.metadata.to_account_info())
            .owner(&ctx.accounts.owner.to_account_info())
            .mint(&ctx.accounts.mint.to_account_info())
            .token_account(&ctx.accounts.owner_item_token_account.to_account_info())
            .master_edition_account(&ctx.accounts.master_edition.to_account_info())
            .spl_token_program(&ctx.accounts.token_program.to_account_info())
            .invoke()
            .map_err(anchor_lang::error::Error::from)?;

        Ok(())
    }
}

/// Accounts required to mint a crafted item NFT.
#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct MintItemNft<'info> {
    /// Current player paying rent for metadata-related accounts.
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump = game_config.bump,
        constraint = usize::from(item_type) < ITEM_COUNT @ GameErrorCode::InvalidItemTypeIndex
    )]
    /// Shared config used to validate the item type index.
    pub game_config: Account<'info, GameConfig>,
    #[account(
        signer @ GameErrorCode::UnauthorizedItemMintCaller,
        seeds = [PROGRAM_AUTHORITY_SEED],
        seeds::program = CRAFTING_PROGRAM_ID,
        bump
    )]
    /// CHECK: this PDA is derived from the crafting program and must be signed via CPI.
    /// Crafting-program PDA proving the CPI caller is authorized.
    pub caller_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: this PDA signs Metaplex CPIs as mint/update authority.
    /// Item-nft PDA acting as mint and update authority in Metaplex.
    pub program_authority: UncheckedAccount<'info>,
    /// Fresh NFT mint signer created off-chain for this item.
    #[account(mut)]
    pub mint: Signer<'info>,
    #[account(
        init,
        payer = owner,
        space = 8 + ItemMetadata::INIT_SPACE,
        seeds = [ITEM_METADATA_SEED, mint.key().as_ref()],
        bump
    )]
    /// PDA storing item-type and owner data for the minted NFT.
    pub item_metadata: Account<'info, ItemMetadata>,
    #[account(mut)]
    /// CHECK: validated against the Metaplex PDA derivation before CPI.
    /// Canonical Metaplex metadata PDA for the NFT.
    pub metadata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: validated against the Metaplex PDA derivation before CPI.
    /// Canonical Metaplex master edition PDA for the NFT.
    pub master_edition: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: validated as the expected ATA for the owner and mint before CPI.
    /// Owner ATA that receives the newly minted NFT.
    pub owner_item_token_account: UncheckedAccount<'info>,
    #[account(address = mpl_token_metadata::ID)]
    /// CHECK: constrained to the canonical Metaplex Token Metadata program.
    /// Canonical Metaplex Token Metadata program.
    pub token_metadata_program: UncheckedAccount<'info>,
    /// SPL Token program that owns the NFT mint and ATA.
    pub token_program: Program<'info, Token>,
    /// Associated token program used by Metaplex mint flows.
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// System program used by Metaplex CPIs.
    pub system_program: Program<'info, System>,
    #[account(address = SYSVAR_INSTRUCTIONS_ID)]
    /// CHECK: constrained to the instructions sysvar address.
    /// Instructions sysvar required by Metaplex CPIs.
    pub sysvar_instructions: UncheckedAccount<'info>,
}

/// Accounts required to burn a redeemed item NFT.
#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct BurnItemNft<'info> {
    /// Current NFT owner authorizing the burn.
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump = game_config.bump,
        constraint = usize::from(item_type) < ITEM_COUNT @ GameErrorCode::InvalidItemTypeIndex
    )]
    /// Shared config used to validate the item type index.
    pub game_config: Account<'info, GameConfig>,
    #[account(
        signer @ GameErrorCode::UnauthorizedItemBurnCaller,
        seeds = [PROGRAM_AUTHORITY_SEED],
        seeds::program = MARKETPLACE_PROGRAM_ID,
        bump
    )]
    /// CHECK: this PDA is derived from the marketplace program and must be signed via CPI.
    /// Marketplace-program PDA proving the CPI caller is authorized.
    pub caller_authority: UncheckedAccount<'info>,
    /// NFT mint being burned.
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        close = owner,
        seeds = [ITEM_METADATA_SEED, mint.key().as_ref()],
        bump = item_metadata.bump
    )]
    /// Metadata account that is closed once the NFT is successfully burned.
    pub item_metadata: Account<'info, ItemMetadata>,
    #[account(mut)]
    /// CHECK: validated against the Metaplex PDA derivation before CPI.
    /// Canonical Metaplex metadata PDA for the NFT.
    pub metadata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: validated against the Metaplex PDA derivation before CPI.
    /// Canonical Metaplex master edition PDA for the NFT.
    pub master_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = owner,
        token::token_program = token_program
    )]
    /// Owner ATA currently holding the NFT token.
    pub owner_item_token_account: Account<'info, TokenAccount>,
    #[account(address = mpl_token_metadata::ID)]
    /// CHECK: constrained to the canonical Metaplex Token Metadata program.
    /// Canonical Metaplex Token Metadata program.
    pub token_metadata_program: UncheckedAccount<'info>,
    /// SPL Token program used to burn the NFT.
    pub token_program: Program<'info, Token>,
    /// System program required by Metaplex during burn cleanup.
    pub system_program: Program<'info, System>,
    #[account(address = SYSVAR_INSTRUCTIONS_ID)]
    /// CHECK: constrained to the instructions sysvar address.
    /// Instructions sysvar required by Metaplex CPIs.
    pub sysvar_instructions: UncheckedAccount<'info>,
}

/// Per-item metadata tracked outside Metaplex for gameplay validation.
#[account]
#[derive(InitSpace)]
pub struct ItemMetadata {
    /// Stored item type discriminator.
    pub item_type: u8,
    /// Wallet that owned the NFT when it was minted.
    pub owner: Pubkey,
    /// NFT mint associated with this metadata account.
    pub mint: Pubkey,
    /// PDA bump for the metadata account.
    pub bump: u8,
}

impl ItemMetadata {
    /// Returns signer seeds for the item metadata PDA when needed by CPI helpers.
    pub fn signer_seeds<'a>(mint: &'a Pubkey, bump: &'a u8) -> [&'a [u8]; 3] {
        [
            ITEM_METADATA_SEED,
            mint.as_ref(),
            core::slice::from_ref(bump),
        ]
    }
}

/// Parses and validates an item-type index coming from instruction data.
fn parse_item_type(item_type: u8) -> Result<ItemType> {
    ItemType::from_index(usize::from(item_type))
}

/// Verifies that the provided metadata and edition accounts match Metaplex PDAs.
fn assert_metaplex_accounts(
    mint: &Pubkey,
    metadata: &Pubkey,
    master_edition: &Pubkey,
) -> Result<()> {
    let expected_metadata = Metadata::find_pda(mint).0;
    require_keys_eq!(
        *metadata,
        expected_metadata,
        GameErrorCode::InvalidMetaplexMetadataAddress
    );

    let expected_master_edition = MasterEdition::find_pda(mint).0;
    require_keys_eq!(
        *master_edition,
        expected_master_edition,
        GameErrorCode::InvalidMetaplexMasterEditionAddress
    );

    Ok(())
}
