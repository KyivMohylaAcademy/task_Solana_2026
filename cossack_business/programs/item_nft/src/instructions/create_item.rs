use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::{ItemNftConfig, ItemMetadata};
use crate::constants::MPL_TOKEN_METADATA_ID;

/// Accounts for CPI-gated NFT creation (crafting program only).
#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CreateItem<'info> {
    /// CPI authority from crafting program
    #[account(
        seeds = [b"caller_authority"],
        bump,
        seeds::program = config.crafting_program,
    )]
    pub caller_authority: Signer<'info>,

    #[account(seeds = [b"item_nft_config"], bump = config.bump)]
    pub config: Account<'info, ItemNftConfig>,

    /// CHECK: PDA used as NFT authority
    #[account(seeds = [b"nft_authority"], bump = config.nft_authority_bump)]
    pub nft_authority: UncheckedAccount<'info>,

    /// CHECK: The player receiving the NFT
    pub player: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = nft_authority,
        mint::freeze_authority = nft_authority,
    )]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = player,
    )]
    pub player_nft_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        space = 8 + ItemMetadata::INIT_SPACE,
        seeds = [b"item_metadata", nft_mint.key().as_ref()],
        bump,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// CHECK: Metaplex metadata PDA (validated by Metaplex CPI)
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK: Metaplex master edition PDA (validated by Metaplex CPI)
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: Metaplex Token Metadata program
    #[account(address = MPL_TOKEN_METADATA_ID)]
    pub metadata_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
