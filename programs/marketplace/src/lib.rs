use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use item_nft::cpi::accounts::BurnItem;
use item_nft::program::ItemNft;
use item_nft::ItemMetadata;
use magic_token::cpi::accounts::MintMagicToken;
use magic_token::program::MagicToken;
use magic_token::MagicTokenConfig;

declare_id!("GvoJJmQiBXdLQrzP33uhiyQWxBPo254qp8iuotXR9Eeu");

pub const ITEM_PRICES: [u64; 4] = [100, 150, 200, 300];

#[program]
pub mod marketplace {
    use super::*;

    pub fn sell_item(ctx: Context<SellItem>) -> Result<()> {
        let item_type = ctx.accounts.item_metadata.item_type;
        require!(
            (item_type as usize) < ITEM_PRICES.len(),
            MarketplaceError::InvalidItemType
        );

        let price = ITEM_PRICES[item_type as usize];

        require!(
            ctx.accounts.player_item_account.amount >= 1,
            MarketplaceError::NoItemToSell
        );

        let burn_accounts = BurnItem {
            item_metadata: ctx.accounts.item_metadata.to_account_info(),
            item_mint: ctx.accounts.item_mint.to_account_info(),
            player_item_account: ctx.accounts.player_item_account.to_account_info(),
            player: ctx.accounts.player.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        let burn_ctx = CpiContext::new(
            ctx.accounts.item_nft_program.to_account_info(),
            burn_accounts,
        );

        item_nft::cpi::burn_item(burn_ctx)?;

        let mint_accounts = MintMagicToken {
            magic_token_config: ctx.accounts.magic_token_config.to_account_info(),
            magic_token_mint: ctx.accounts.magic_token_mint.to_account_info(),
            mint_authority: ctx.accounts.magic_mint_authority.to_account_info(),
            player_token_account: ctx.accounts.player_magic_account.to_account_info(),
            token_program: ctx.accounts.token_2022_program.to_account_info(),
        };

        let mint_ctx = CpiContext::new(
            ctx.accounts.magic_token_program.to_account_info(),
            mint_accounts,
        );

        magic_token::cpi::mint_magic_token(mint_ctx, price)?;

        msg!(
            "Marketplace: Sold item type {} for {} MagicToken",
            item_type,
            price
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SellItem<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        constraint = item_metadata.owner == player.key() @ MarketplaceError::NotItemOwner,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    #[account(mut, address = item_metadata.mint)]
    pub item_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = item_mint,
        associated_token::authority = player,
    )]
    pub player_item_account: InterfaceAccount<'info, TokenAccount>,

    pub magic_token_config: Account<'info, MagicTokenConfig>,

    #[account(mut, address = magic_token_config.mint)]
    pub magic_token_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: PDA mint authority
    pub magic_mint_authority: AccountInfo<'info>,

    #[account(
        mut,
        token::mint = magic_token_mint,
        token::authority = player,
    )]
    pub player_magic_account: InterfaceAccount<'info, TokenAccount>,

    pub item_nft_program: Program<'info, ItemNft>,
    pub magic_token_program: Program<'info, MagicToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_2022_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum MarketplaceError {
    #[msg("Invalid item type.")]
    InvalidItemType,
    #[msg("No item to sell. Token account is empty.")]
    NoItemToSell,
    #[msg("Not the owner of this item.")]
    NotItemOwner,
}
