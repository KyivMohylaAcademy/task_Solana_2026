use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token::{self, MintTo};

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("JBPwQkcUjqGP2mJEhsb7BeNGDTptkTn8968ZEidb4qDg");

#[program]
pub mod item_nft {
    use super::*;

    /// Store authorized caller program IDs.
    pub fn initialize_item_nft(
        ctx: Context<InitializeItemNft>,
        crafting_program: Pubkey,
        marketplace_program: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.crafting_program = crafting_program;
        config.marketplace_program = marketplace_program;
        config.bump = ctx.bumps.config;
        config.nft_authority_bump = ctx.bumps.nft_authority;
        Ok(())
    }

    /// Create an NFT item. CPI-gated: only callable by the crafting program.
    pub fn create_item(ctx: Context<CreateItem>, item_type: u8) -> Result<()> {
        require!(item_type < ITEM_COUNT as u8, ItemError::InvalidItemType);

        let nft_bump = ctx.accounts.config.nft_authority_bump;
        let nft_seeds: &[&[u8]] = &[b"nft_authority", &[nft_bump]];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    to: ctx.accounts.player_nft_ata.to_account_info(),
                    authority: ctx.accounts.nft_authority.to_account_info(),
                },
                &[nft_seeds],
            ),
            1,
        )?;

        let name = ITEM_NAMES[item_type as usize].to_string();
        let symbol = ITEM_SYMBOLS[item_type as usize].to_string();
        let uri = String::new();

        let metadata_accounts = vec![
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.nft_authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        let create_metadata_ix = mpl_token_metadata::instructions::CreateMetadataAccountV3Builder::new()
            .metadata(ctx.accounts.metadata_account.key())
            .mint(ctx.accounts.nft_mint.key())
            .mint_authority(ctx.accounts.nft_authority.key())
            .payer(ctx.accounts.payer.key())
            .update_authority(ctx.accounts.nft_authority.key(), true)
            .system_program(ctx.accounts.system_program.key())
            .rent(Some(ctx.accounts.rent.key()))
            .data(mpl_token_metadata::types::DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            })
            .is_mutable(true)
            .instruction();

        invoke_signed(&create_metadata_ix, &metadata_accounts, &[nft_seeds])?;

        let edition_accounts = vec![
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        let create_edition_ix = mpl_token_metadata::instructions::CreateMasterEditionV3Builder::new()
            .edition(ctx.accounts.master_edition.key())
            .mint(ctx.accounts.nft_mint.key())
            .update_authority(ctx.accounts.nft_authority.key())
            .mint_authority(ctx.accounts.nft_authority.key())
            .payer(ctx.accounts.payer.key())
            .metadata(ctx.accounts.metadata_account.key())
            .token_program(ctx.accounts.token_program.key())
            .system_program(ctx.accounts.system_program.key())
            .rent(Some(ctx.accounts.rent.key()))
            .max_supply(0)
            .instruction();

        invoke_signed(&create_edition_ix, &edition_accounts, &[nft_seeds])?;

        let item_meta = &mut ctx.accounts.item_metadata;
        item_meta.item_type = item_type;
        item_meta.owner = ctx.accounts.player.key();
        item_meta.mint = ctx.accounts.nft_mint.key();
        item_meta.bump = ctx.bumps.item_metadata;

        Ok(())
    }

    /// Burn an NFT item. CPI-gated: only callable by the marketplace program.
    pub fn burn_item(ctx: Context<BurnItem>) -> Result<()> {
        let nft_bump = ctx.accounts.config.nft_authority_bump;
        let nft_seeds: &[&[u8]] = &[b"nft_authority", &[nft_bump]];

        let burn_accounts = vec![
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.player.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.player_nft_ata.to_account_info(),
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
        ];

        let burn_ix = mpl_token_metadata::instructions::BurnV1Builder::new()
            .authority(ctx.accounts.player.key())
            .metadata(ctx.accounts.metadata_account.key())
            .edition(Some(ctx.accounts.master_edition.key()))
            .mint(ctx.accounts.nft_mint.key())
            .token(ctx.accounts.player_nft_ata.key())
            .spl_token_program(ctx.accounts.token_program.key())
            .system_program(ctx.accounts.system_program.key())
            .sysvar_instructions(ctx.accounts.sysvar_instructions.key())
            .instruction();

        invoke_signed(&burn_ix, &burn_accounts, &[nft_seeds])?;

        Ok(())
    }

    /// Transfer ownership of an item's metadata. CPI-gated: only marketplace.
    pub fn transfer_item_ownership(
        ctx: Context<TransferItemOwnership>,
        new_owner: Pubkey,
    ) -> Result<()> {
        ctx.accounts.item_metadata.owner = new_owner;
        Ok(())
    }
}
