use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount};
use item_nft::cpi::accounts::BurnItemNft;
use item_nft::cpi::burn_item_nft;
use item_nft::ItemCollection;
use item_nft::ItemMetadata;
use magic_token::cpi::accounts::MintMagicTokens;
use magic_token::cpi::mint_magic_tokens;
use magic_token::MagicTokenConfig;
use resource_manager::GameConfig;

declare_id!("FAuToaomcNLiYTJFVoqTiGHqakDBC3T78MNfvCiHjfNh");

/// Marketplace — програма для продажу NFT-предметів за MagicToken.
///
/// Гравці можуть продавати свої NFT-предмети на маркетплейсі.
/// При продажу NFT спалюється, а гравець отримує MagicTokens
/// відповідно до ціни предмета з GameConfig.
#[program]
pub mod marketplace {
    use super::*;

    /// Продаж NFT-предмета на маркетплейсі.
    ///
    /// Спалює NFT через CPI до item_nft::burn_item_nft та
    /// мінтить MagicTokens гравцю через CPI до magic_token::mint_magic_tokens.
    /// Ціна визначається game_config.item_prices[item_type].
    pub fn sell_item(ctx: Context<SellItem>, item_type: u8) -> Result<()> {
        require!(item_type <= 3, MarketplaceError::InvalidItemType);

        // Перевірка відповідності item_metadata та NFT
        require!(
            ctx.accounts.item_metadata.mint == ctx.accounts.nft_mint.key(),
            MarketplaceError::ItemMintMismatch
        );
        require!(
            ctx.accounts.item_metadata.item_type == item_type,
            MarketplaceError::ItemTypeMismatch
        );

        // Отримуємо ціну предмета
        let price = ctx.accounts.game_config.item_prices[item_type as usize];
        msg!(
            "Selling item type {} for {} MagicTokens",
            item_type,
            price
        );

        // CPI authority seeds
        let cpi_bump = ctx.bumps.cpi_authority;
        let cpi_seeds: &[&[u8]] = &[b"cpi_authority", &[cpi_bump]];
        let signer_seeds = &[cpi_seeds];

        // 1. Спалюємо NFT через CPI до item_nft
        let burn_accounts = BurnItemNft {
            collection: ctx.accounts.item_collection.to_account_info(),
            owner: ctx.accounts.seller.to_account_info(),
            nft_mint: ctx.accounts.nft_mint.to_account_info(),
            nft_token_account: ctx.accounts.nft_token_account.to_account_info(),
            item_metadata: ctx.accounts.item_metadata.to_account_info(),
            caller_auth: ctx.accounts.cpi_authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        burn_item_nft(CpiContext::new_with_signer(
            ctx.accounts.item_nft_program.to_account_info(),
            burn_accounts,
            signer_seeds,
        ))?;

        msg!("NFT burned successfully. Minting MagicTokens...");

        // 2. Мінтимо MagicTokens продавцю через CPI до magic_token
        let mint_accounts = MintMagicTokens {
            config: ctx.accounts.magic_config.to_account_info(),
            magic_mint: ctx.accounts.magic_mint.to_account_info(),
            recipient_token_account: ctx.accounts.seller_magic_ata.to_account_info(),
            mint_authority: ctx.accounts.magic_mint_authority.to_account_info(),
            caller_auth: ctx.accounts.cpi_authority.to_account_info(),
            token_program: ctx.accounts.token_2022_program.to_account_info(),
        };

        mint_magic_tokens(
            CpiContext::new_with_signer(
                ctx.accounts.magic_token_program.to_account_info(),
                mint_accounts,
                signer_seeds,
            ),
            price,
        )?;

        msg!("Sold item for {} MagicTokens!", price);
        Ok(())
    }
}

/// Акаунти для продажу предмета.
#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct SellItem<'info> {
    /// Продавець — підписант та власник NFT
    #[account(mut)]
    pub seller: Signer<'info>,

    // === Game Config ===

    /// Конфігурація гри (для item_prices)
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager::ID,
    )]
    pub game_config: Account<'info, GameConfig>,

    // === Item NFT accounts ===

    /// Колекція предметів (ItemCollection PDA)
    #[account(
        seeds = [b"item_collection"],
        bump = item_collection.bump,
        seeds::program = item_nft::ID,
    )]
    pub item_collection: Account<'info, ItemCollection>,

    /// NFT mint
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,

    /// Token account продавця для NFT
    #[account(
        mut,
        constraint = nft_token_account.owner == seller.key(),
        constraint = nft_token_account.mint == nft_mint.key(),
        constraint = nft_token_account.amount == 1,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    /// ItemMetadata PDA
    #[account(
        mut,
        seeds = [b"item_metadata", nft_mint.key().as_ref()],
        bump = item_metadata.bump,
        seeds::program = item_nft::ID,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    // === Magic Token accounts ===

    /// Конфігурація MagicToken
    #[account(
        seeds = [b"magic_config"],
        bump = magic_config.bump,
        seeds::program = magic_token::ID,
    )]
    pub magic_config: Account<'info, MagicTokenConfig>,

    /// MagicToken mint authority PDA
    /// CHECK: PDA magic_token з seeds [b"magic_mint_authority"]
    #[account(
        seeds = [b"magic_mint_authority"],
        bump = magic_config.mint_authority_bump,
        seeds::program = magic_token::ID,
    )]
    pub magic_mint_authority: UncheckedAccount<'info>,

    /// MagicToken mint (Token-2022)
    /// CHECK: Перевіряється magic_config.mint
    #[account(
        mut,
        constraint = magic_mint.key() == magic_config.mint @ MarketplaceError::InvalidItemType,
    )]
    pub magic_mint: UncheckedAccount<'info>,

    /// Token account продавця для MagicToken (Token-2022 ATA)
    /// CHECK: Валідується token програмою під час CPI
    #[account(mut)]
    pub seller_magic_ata: UncheckedAccount<'info>,

    // === CPI Authority ===

    /// CPI authority PDA від marketplace
    /// CHECK: PDA цієї програми з seeds [b"cpi_authority"]
    #[account(
        seeds = [b"cpi_authority"],
        bump,
    )]
    pub cpi_authority: UncheckedAccount<'info>,

    // === Programs ===

    /// Item NFT програма
    /// CHECK: Перевіряється за program ID
    pub item_nft_program: UncheckedAccount<'info>,

    /// Magic token програма
    /// CHECK: Перевіряється за program ID
    pub magic_token_program: UncheckedAccount<'info>,

    /// Token програма (для NFT операцій)
    pub token_program: Program<'info, Token>,

    /// Token-2022 програма (для MagicToken операцій)
    /// CHECK: SPL Token-2022 program
    pub token_2022_program: UncheckedAccount<'info>,

    /// Системна програма
    pub system_program: Program<'info, System>,
}

/// Коди помилок для програми marketplace.
#[error_code]
pub enum MarketplaceError {
    /// Невірний тип предмета (має бути 0-3)
    #[msg("Invalid item type (must be 0-3)")]
    InvalidItemType,
    /// Мінт предмета не відповідає метаданим
    #[msg("Item mint does not match item metadata")]
    ItemMintMismatch,
    /// Тип предмета не відповідає
    #[msg("Item type does not match")]
    ItemTypeMismatch,
}
