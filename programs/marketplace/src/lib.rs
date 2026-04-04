use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint as TokenMint, TokenAccount as Token2022Account};
use game_common::{item_price, GAME_CONFIG_SEED, MARKETPLACE_AUTHORITY_SEED};

declare_id!("E1nMz6JbstqDK9cEFhx1g3XrAJK8J2d9kvGiZTdYVaK9");

#[program]
pub mod marketplace {
    use super::*;

    /// Sells an item to the marketplace sink, burns the NFT and mints MagicToken to the seller.
    pub fn sell_item(ctx: Context<SellItem>) -> Result<()> {
        let reward = item_price(
            ctx.accounts.item_metadata.item_type,
            &ctx.accounts.game_config.item_prices,
        )
        .expect("item type is validated when the NFT is minted");
        let game_config_key = ctx.accounts.game_config.key();

        let marketplace_signer: &[&[u8]] = &[
            MARKETPLACE_AUTHORITY_SEED,
            game_config_key.as_ref(),
            &[ctx.bumps.marketplace_authority],
        ];

        item_nft::cpi::burn_item(CpiContext::new_with_signer(
            ctx.accounts.item_nft_program.to_account_info(),
            item_nft::cpi::accounts::BurnItem {
                owner: ctx.accounts.seller.to_account_info(),
                game_config: ctx.accounts.game_config.to_account_info(),
                marketplace_authority: ctx.accounts.marketplace_authority.to_account_info(),
                item_authority: ctx.accounts.item_authority.to_account_info(),
                item_mint: ctx.accounts.item_mint.to_account_info(),
                owner_item_account: ctx.accounts.seller_item_account.to_account_info(),
                item_metadata: ctx.accounts.item_metadata.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                master_edition: ctx.accounts.master_edition.to_account_info(),
                metadata_program: ctx.accounts.metadata_program.to_account_info(),
                token_program: ctx.accounts.nft_token_program.to_account_info(),
            },
            &[marketplace_signer],
        ))?;

        magic_token::cpi::mint_reward(
            CpiContext::new_with_signer(
                ctx.accounts.magic_token_program.to_account_info(),
                magic_token::cpi::accounts::MintReward {
                    marketplace_authority: ctx.accounts.marketplace_authority.to_account_info(),
                    game_config: ctx.accounts.game_config.to_account_info(),
                    magic_mint: ctx.accounts.magic_mint.to_account_info(),
                    destination: ctx.accounts.seller_magic_account.to_account_info(),
                    mint_authority: ctx.accounts.magic_authority.to_account_info(),
                    token_program: ctx.accounts.token_2022_program.to_account_info(),
                },
                &[marketplace_signer],
            ),
            reward,
        )
    }
}

#[derive(Accounts)]
pub struct SellItem<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID
    )]
    pub game_config: Account<'info, resource_manager::GameConfig>,
    /// CHECK: PDA signer that authorizes marketplace CPI calls.
    #[account(seeds = [MARKETPLACE_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub marketplace_authority: UncheckedAccount<'info>,
    pub item_nft_program: Program<'info, item_nft::program::ItemNft>,
    /// CHECK: PDA validated by seeds against item_nft and used as the Metaplex freeze delegate.
    #[account(mut, seeds = [game_common::ITEM_AUTHORITY_SEED, game_config.key().as_ref()], bump, seeds::program = item_nft::ID)]
    pub item_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub item_mint: Account<'info, Mint>,
    #[account(mut)]
    pub seller_item_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub item_metadata: Account<'info, item_nft::ItemMetadata>,
    /// CHECK: PDA address is validated inside item_nft against Metaplex derivation.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: PDA address is validated inside item_nft against Metaplex derivation.
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: Fixed Metaplex Token Metadata program id.
    #[account(address = metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,
    pub nft_token_program: Program<'info, Token>,
    pub magic_token_program: Program<'info, magic_token::program::MagicToken>,
    #[account(mut)]
    pub magic_mint: InterfaceAccount<'info, TokenMint>,
    /// CHECK: PDA validated by seeds against the magic_token program.
    #[account(mut, seeds = [game_common::MAGIC_AUTHORITY_SEED, game_config.key().as_ref()], bump, seeds::program = magic_token::ID)]
    pub magic_authority: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = seller,
        associated_token::mint = magic_mint,
        associated_token::authority = seller,
        associated_token::token_program = token_2022_program
    )]
    pub seller_magic_account: InterfaceAccount<'info, Token2022Account>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
