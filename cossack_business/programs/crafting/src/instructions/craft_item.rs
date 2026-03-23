use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token_interface::TokenInterface;
use item_nft::program::ItemNft;
use resource_manager::program::ResourceManager;
use resource_manager::{self as rm};

/// Accounts for crafting an item by burning resources and minting an NFT.
#[derive(Accounts)]
pub struct CraftItem<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    /// CHECK: PDA of this program
    #[account(seeds = [b"caller_authority"], bump)]
    pub caller_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub game_config: Account<'info, rm::GameConfig>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub item_nft_program: Program<'info, ItemNft>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
