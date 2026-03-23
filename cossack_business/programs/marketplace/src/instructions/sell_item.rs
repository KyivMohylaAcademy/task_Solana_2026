use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::token_interface::{
    Mint as MintInterface, TokenAccount as TokenAccountInterface, TokenInterface,
};
use item_nft::program::ItemNft;
use magic_token::program::MagicToken;
use resource_manager::{self as rm};
use crate::errors::MarketError;

/// Accounts for selling an item to the game at a fixed price (burn NFT, mint MagicToken).
#[derive(Accounts)]
pub struct SellItem<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    /// CHECK: This program's CPI authority PDA
    #[account(seeds = [b"caller_authority"], bump)]
    pub caller_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager::ID,
    )]
    pub game_config: Box<Account<'info, rm::GameConfig>>,

    #[account(mut)]
    pub item_metadata: Box<Account<'info, item_nft::ItemMetadata>>,

    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = nft_mint,
        token::authority = seller,
    )]
    pub seller_nft_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"item_nft_config"],
        bump = item_nft_config.bump,
        seeds::program = item_nft_program.key(),
    )]
    pub item_nft_config: Box<Account<'info, item_nft::ItemNftConfig>>,

    #[account(
        seeds = [b"magic_config"],
        bump = magic_token_config.bump,
        seeds::program = magic_token_program.key(),
    )]
    pub magic_token_config: Box<Account<'info, magic_token::MagicTokenConfig>>,

    #[account(
        mut,
        constraint = magic_token_mint.key() == magic_token_config.mint @ MarketError::MintMismatch,
    )]
    pub magic_token_mint: InterfaceAccount<'info, MintInterface>,

    /// CHECK: magic_token mint authority PDA
    #[account(
        seeds = [b"magic_mint_authority"],
        bump = magic_token_config.mint_authority_bump,
        seeds::program = magic_token_program.key(),
    )]
    pub magic_mint_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub seller_magic_ata: InterfaceAccount<'info, TokenAccountInterface>,

    pub item_nft_program: Program<'info, ItemNft>,
    pub magic_token_program: Program<'info, MagicToken>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
