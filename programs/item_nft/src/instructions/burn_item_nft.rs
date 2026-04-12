use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use mpl_token_metadata::accounts::{MasterEdition, Metadata};
use mpl_token_metadata::instructions::BurnV1Builder;

use crate::errors::ItemNftError;
use crate::state::ItemMetadata;

use super::create_item_nft::ITEM_METADATA_SEED;

/// Burn NFT предмета. Тільки через CPI від marketplace (buy_item).
/// Marketplace передає свою PDA як підписувача.
pub const MARKETPLACE_AUTHORITY_SEED: &[u8] = b"marketplace_authority";

#[derive(Accounts)]
pub struct BurnItemNft<'info> {
    /// Гравець — власник NFT. Підписує транзакцію (або marketplace CPI signer).
    #[account(mut)]
    pub authority: Signer<'info>,

    /// ItemMetadata PDA.
    #[account(
        mut,
        seeds = [ITEM_METADATA_SEED, nft_mint.key().as_ref()],
        bump = item_metadata.bump,
        constraint = item_metadata.mint == nft_mint.key(),
        close = authority,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// NFT мінт.
    #[account(mut)]
    pub nft_mint: InterfaceAccount<'info, Mint>,

    /// Токен-акаунт з NFT.
    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = authority,
        token::token_program = token_program,
    )]
    pub nft_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Metaplex Metadata акаунт.
    /// CHECK: Адреса перевіряється через Metaplex PDA.
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
    pub system_program: Program<'info, System>,

    /// CHECK: Sysvar instructions — Sysvar1nstructions1111111111111111111111111.
    pub sysvar_instructions: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<BurnItemNft>) -> Result<()> {
    // Metaplex BurnV1 CPI — спалює і мінт, і метадані
    let burn_ix = BurnV1Builder::new()
        .authority(ctx.accounts.authority.key())
        .metadata(ctx.accounts.metadata_account.key())
        .edition(Some(ctx.accounts.master_edition.key()))
        .mint(ctx.accounts.nft_mint.key())
        .token(ctx.accounts.nft_token_account.key())
        .system_program(ctx.accounts.system_program.key())
        .sysvar_instructions(ctx.accounts.sysvar_instructions.key())
        .spl_token_program(ctx.accounts.token_program.key())
        .instruction();

    anchor_lang::solana_program::program::invoke(
        &burn_ix,
        &[
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.nft_token_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.sysvar_instructions.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
    )?;

    msg!("NFT спалено. Mint: {}", ctx.accounts.nft_mint.key());
    Ok(())
}
