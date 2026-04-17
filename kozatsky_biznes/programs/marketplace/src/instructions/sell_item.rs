use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
};
use resource_manager::{
    self,
    program::ResourceManager,
    state::GameConfig,
};
use item_nft::{
    self,
    cpi::accounts::BurnNftItem,
    program::ItemNft,
    state::{ItemNftConfig, ItemMetadata},
};
use magic_token::{
    self,
    cpi::accounts::MintMagicToken,
    program::MagicToken,
    state::{MagicTokenConfig, MintAuthority as MtMintAuthority},
};

use crate::state::*;
use crate::error::MarketplaceError;
use crate::constants::ITEM_COUNT;

/// Продає NFT предмет гравця за MagicToken.
///
/// Процес:
/// 1. Перевіряє тип предмету через ItemMetadata
/// 2. Отримує ціну з resource_manager GameConfig.item_prices[item_type]
/// 3. Спалює NFT через CPI до item_nft::burn_nft_item
/// 4. Мінтить MagicToken через CPI до magic_token::mint_magic_token
#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct SellItem<'info> {
    /// Конфігурація гри (resource_manager) — для отримання ціни предмету
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub game_config: Account<'info, GameConfig>,

    /// MarketplaceAuthority PDA (підписує CPI)
    #[account(
        seeds = [b"marketplace_authority"],
        bump = marketplace_authority.bump,
    )]
    pub marketplace_authority: Account<'info, MarketplaceAuthority>,

    /// Конфігурація ItemNft
    #[account(
        seeds = [b"item_nft_config"],
        bump = item_nft_config.bump,
        seeds::program = item_nft_program.key(),
    )]
    pub item_nft_config: Account<'info, ItemNftConfig>,

    /// Конфігурація MagicToken
    #[account(
        seeds = [b"magic_token_config"],
        bump = magic_token_config.bump,
        seeds::program = magic_token_program.key(),
    )]
    pub magic_token_config: Account<'info, MagicTokenConfig>,

    /// PDA-авторитет мінту MagicToken
    #[account(
        seeds = [b"mt_mint_authority"],
        bump = mt_mint_authority.bump,
        seeds::program = magic_token_program.key(),
    )]
    pub mt_mint_authority: Account<'info, MtMintAuthority>,

    /// Гравець (підписант, власник NFT)
    #[account(mut)]
    pub player: Signer<'info>,

    /// NFT мінт предмету
    /// CHECK: Перевіряється через item_metadata.mint
    #[account(
        mut,
        constraint = nft_mint.key() == item_metadata.mint @ MarketplaceError::MintMismatch
    )]
    pub nft_mint: AccountInfo<'info>,

    /// Метадані предмету (для перевірки типу)
    #[account(
        mut,
        seeds = [b"item_metadata", nft_mint.key().as_ref()],
        bump = item_metadata.bump,
        seeds::program = item_nft_program.key(),
        constraint = item_metadata.item_type == item_type @ MarketplaceError::ItemTypeMismatch,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// Токен-акаунт гравця з NFT
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = player,
        associated_token::token_program = token_2022_program,
    )]
    pub player_nft_token_account: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    /// Мінт MagicToken
    /// CHECK: Перевіряється через magic_token_config.mint
    #[account(
        mut,
        constraint = magic_token_mint.key() == magic_token_config.mint @ MarketplaceError::PriceNotSet
    )]
    pub magic_token_mint: AccountInfo<'info>,

    /// Асоційований токен-акаунт гравця для MagicToken (ініціалізується якщо не існує)
    #[account(
        init_if_needed,
        payer = player,
        associated_token::mint = magic_token_mint,
        associated_token::authority = player,
        associated_token::token_program = token_2022_program,
    )]
    pub player_magic_token_account: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub item_nft_program: Program<'info, ItemNft>,
    pub magic_token_program: Program<'info, MagicToken>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SellItem>, item_type: u8) -> Result<()> {
    require!(item_type < ITEM_COUNT as u8, MarketplaceError::InvalidItemType);

    // Отримуємо ціну предмету з конфігурації гри
    let price = ctx.accounts.game_config.item_prices[item_type as usize];
    require!(price > 0, MarketplaceError::PriceNotSet);

    let marketplace_bump = ctx.accounts.marketplace_authority.bump;
    let marketplace_seeds: &[&[u8]] = &[b"marketplace_authority", &[marketplace_bump]];

    // CPI до item_nft::burn_nft_item — спалюємо NFT
    item_nft::cpi::burn_nft_item(
        CpiContext::new_with_signer(
            ctx.accounts.item_nft_program.key(),
            BurnNftItem {
                item_nft_config: ctx.accounts.item_nft_config.to_account_info(),
                caller_authority: ctx.accounts.marketplace_authority.to_account_info(),
                player: ctx.accounts.player.to_account_info(),
                nft_mint: ctx.accounts.nft_mint.to_account_info(),
                item_metadata: ctx.accounts.item_metadata.to_account_info(),
                player_nft_token_account: ctx.accounts.player_nft_token_account.to_account_info(),
                token_2022_program: ctx.accounts.token_2022_program.to_account_info(),
            },
            &[marketplace_seeds],
        ),
    )?;

    // CPI до magic_token::mint_magic_token — мінтимо MagicToken гравцю
    magic_token::cpi::mint_magic_token(
        CpiContext::new_with_signer(
            ctx.accounts.magic_token_program.key(),
            MintMagicToken {
                magic_token_config: ctx.accounts.magic_token_config.to_account_info(),
                mint_authority: ctx.accounts.mt_mint_authority.to_account_info(),
                caller_authority: ctx.accounts.marketplace_authority.to_account_info(),
                payer: ctx.accounts.player.to_account_info(),
                player: ctx.accounts.player.to_account_info(),
                mint: ctx.accounts.magic_token_mint.to_account_info(),
                player_token_account: ctx.accounts.player_magic_token_account.to_account_info(),
                token_2022_program: ctx.accounts.token_2022_program.to_account_info(),
                associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
            &[marketplace_seeds],
        ),
        price,
    )?;

    msg!(
        "Гравець {} продав предмет типу {} за {} MagicToken",
        ctx.accounts.player.key(),
        item_type,
        price
    );
    Ok(())
}
