use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{close_account, CloseAccount, Mint, TokenAccount};
use item_nft::cpi::accounts::BurnItemNft;
use item_nft::cpi::burn_item_nft;
use item_nft::program::ItemNft;
use magic_token::cpi::accounts::MintMagicToken;
use magic_token::cpi::mint_magic_token;
use magic_token::program::MagicToken;
use magic_token::state::MagicConfig;

use crate::state::Listing;

use super::list_item::{ESCROW_SEED, LISTING_SEED};

pub const MARKETPLACE_AUTHORITY_SEED: &[u8] = b"marketplace_authority";
pub const MAGIC_CONFIG_SEED: &[u8] = b"magic_config";

#[derive(Accounts)]
pub struct BuyItem<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: Адреса перевіряється через listing.seller.
    #[account(mut, constraint = listing.seller == seller.key())]
    pub seller: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [LISTING_SEED, nft_mint.key().as_ref()],
        bump = listing.bump,
        close = seller,
    )]
    pub listing: Box<Account<'info, Listing>>,

    #[account(mut)]
    pub nft_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, nft_mint.key().as_ref()],
        bump,
        token::mint = nft_mint,
        token::authority = listing,
        token::token_program = token_program,
    )]
    pub escrow_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Перевіряється через item_nft CPI.
    #[account(mut)]
    pub item_metadata: UncheckedAccount<'info>,

    /// CHECK: Metaplex Metadata.
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK: Metaplex program.
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,

    #[account(
        seeds = [MAGIC_CONFIG_SEED],
        bump = magic_config.bump,
        seeds::program = magic_token_program.key(),
    )]
    pub magic_config: Box<Account<'info, MagicConfig>>,

    #[account(mut, constraint = magic_config.mint == magic_mint.key())]
    pub magic_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = magic_mint,
        token::authority = seller,
        token::token_program = token_program,
    )]
    pub seller_magic_account: InterfaceAccount<'info, TokenAccount>,

    /// Marketplace authority PDA — mint_authority для MagicToken.
    /// CHECK: Перевіряється через magic_config.marketplace_authority.
    #[account(
        mut,
        seeds = [MARKETPLACE_AUTHORITY_SEED],
        bump,
        constraint = magic_config.marketplace_authority == marketplace_authority.key(),
    )]
    pub marketplace_authority: UncheckedAccount<'info>,

    /// CHECK: Metaplex Master Edition PDA.
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: Sysvar instructions — Sysvar1nstructions1111111111111111111111111.
    pub sysvar_instructions: UncheckedAccount<'info>,

    pub item_nft_program: Program<'info, ItemNft>,
    pub magic_token_program: Program<'info, MagicToken>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyItem>) -> Result<()> {
    let listing_bump = ctx.accounts.listing.bump;
    let nft_mint_key = ctx.accounts.nft_mint.key();
    let listing_seeds: &[&[u8]] = &[LISTING_SEED, nft_mint_key.as_ref(), &[listing_bump]];

    // 1. Спалюємо NFT через item_nft CPI
    burn_item_nft(CpiContext::new_with_signer(
        item_nft::ID,
        BurnItemNft {
            authority: ctx.accounts.listing.to_account_info(),
            item_metadata: ctx.accounts.item_metadata.to_account_info(),
            nft_mint: ctx.accounts.nft_mint.to_account_info(),
            nft_token_account: ctx.accounts.escrow_account.to_account_info(),
            metadata_account: ctx.accounts.metadata_account.to_account_info(),
            master_edition: ctx.accounts.master_edition.to_account_info(),
            token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            sysvar_instructions: ctx.accounts.sysvar_instructions.to_account_info(),
        },
        &[listing_seeds],
    ))?;

    // 2. Мінтимо MagicToken продавцю
    let mp_bump = ctx.bumps.marketplace_authority;
    let mp_seeds: &[&[u8]] = &[MARKETPLACE_AUTHORITY_SEED, &[mp_bump]];
    mint_magic_token(
        CpiContext::new_with_signer(
            magic_token::ID,
            MintMagicToken {
                magic_config: ctx.accounts.magic_config.to_account_info(),
                magic_mint: ctx.accounts.magic_mint.to_account_info(),
                recipient_token_account: ctx.accounts.seller_magic_account.to_account_info(),
                marketplace_authority: ctx.accounts.marketplace_authority.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
            &[mp_seeds],
        ),
        ctx.accounts.listing.price,
    )?;

    msg!(
        "NFT куплено. Продавець {} отримав {} MagicToken",
        ctx.accounts.seller.key(),
        ctx.accounts.listing.price
    );
    Ok(())
}
