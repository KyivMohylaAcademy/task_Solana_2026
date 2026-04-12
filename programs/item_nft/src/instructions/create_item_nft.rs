use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use mpl_token_metadata::accounts::{MasterEdition, Metadata};
use mpl_token_metadata::instructions::{CreateV1Builder, MintV1Builder};
use mpl_token_metadata::types::{PrintSupply, TokenStandard};

use crate::errors::ItemNftError;
use crate::state::ItemMetadata;

pub const ITEM_METADATA_SEED: &[u8] = b"item_metadata";
pub const CRAFTING_AUTHORITY_SEED: &[u8] = b"crafting_authority";

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CreateItemNft<'info> {
    /// Гравець — отримувач NFT і підписувач транзакції (через crafting CPI).
    #[account(mut)]
    pub player: Signer<'info>,

    /// ItemMetadata PDA для цього мінту.
    #[account(
        init,
        payer = player,
        space = ItemMetadata::LEN,
        seeds = [ITEM_METADATA_SEED, nft_mint.key().as_ref()],
        bump,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// NFT мінт — новий акаунт, ініціалізується тут.
    #[account(
        init,
        payer = player,
        mint::decimals = 0,
        mint::authority = player,
        mint::freeze_authority = player,
    )]
    pub nft_mint: InterfaceAccount<'info, Mint>,

    /// Токен-акаунт гравця для цього NFT.
    #[account(
        init,
        payer = player,
        associated_token::mint = nft_mint,
        associated_token::authority = player,
    )]
    pub player_nft_account: InterfaceAccount<'info, TokenAccount>,

    /// Metaplex Metadata акаунт (PDA від Metaplex).
    /// CHECK: Створюється через Metaplex CPI.
    #[account(
        mut,
        address = Metadata::find_pda(&nft_mint.key()).0,
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK: Metaplex Master Edition PDA.
    #[account(
        mut,
        address = MasterEdition::find_pda(&nft_mint.key()).0,
    )]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: Metaplex Token Metadata program.
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

    /// CHECK: Sysvar instructions — Sysvar1nstructions1111111111111111111111111.
    pub sysvar_instructions: UncheckedAccount<'info>,
}

pub fn handler(
    ctx: Context<CreateItemNft>,
    item_type: u8,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    require!(
        item_type < ItemMetadata::ITEM_TYPE_COUNT,
        ItemNftError::InvalidItemType
    );

    // Заповнюємо ItemMetadata
    let meta = &mut ctx.accounts.item_metadata;
    meta.item_type = item_type;
    meta.owner = ctx.accounts.player.key();
    meta.mint = ctx.accounts.nft_mint.key();
    meta.bump = ctx.bumps.item_metadata;

    // Metaplex CreateV1 CPI
    let create_ix = CreateV1Builder::new()
        .metadata(ctx.accounts.metadata_account.key())
        .master_edition(Some(ctx.accounts.master_edition.key()))
        .mint(ctx.accounts.nft_mint.key(), true)
        .authority(ctx.accounts.player.key())
        .payer(ctx.accounts.player.key())
        .update_authority(ctx.accounts.player.key(), true)
        .system_program(ctx.accounts.system_program.key())
        .sysvar_instructions(ctx.accounts.sysvar_instructions.key())
        .spl_token_program(Some(ctx.accounts.token_program.key()))
        .name(name)
        .symbol(symbol)
        .uri(uri)
        .seller_fee_basis_points(0)
        .token_standard(TokenStandard::NonFungible)
        .print_supply(PrintSupply::Zero)
        .instruction();

    anchor_lang::solana_program::program::invoke(
        &create_ix,
        &[
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.player.to_account_info(),
            ctx.accounts.player.to_account_info(),
            ctx.accounts.player.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
    )?;

    // Metaplex MintV1 CPI — мінтить 1 токен у player_nft_account
    let mint_ix = MintV1Builder::new()
        .token(ctx.accounts.player_nft_account.key())
        .token_owner(Some(ctx.accounts.player.key()))
        .metadata(ctx.accounts.metadata_account.key())
        .master_edition(Some(ctx.accounts.master_edition.key()))
        .mint(ctx.accounts.nft_mint.key())
        .payer(ctx.accounts.player.key())
        .authority(ctx.accounts.player.key())
        .system_program(ctx.accounts.system_program.key())
        .sysvar_instructions(ctx.accounts.sysvar_instructions.key())
        .spl_token_program(ctx.accounts.token_program.key())
        .spl_ata_program(ctx.accounts.associated_token_program.key())
        .amount(1)
        .instruction();

    anchor_lang::solana_program::program::invoke(
        &mint_ix,
        &[
            ctx.accounts.player_nft_account.to_account_info(),
            ctx.accounts.player.to_account_info(),
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.player.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.associated_token_program.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
        ],
    )?;

    msg!("NFT предмет типу {} створено. Mint: {}", item_type, ctx.accounts.nft_mint.key());
    Ok(())
}
