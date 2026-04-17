use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, MintTo, Token2022};

declare_id!("FBKAbyCSWv1Vm7PVw1NRGWnfH9rpLXqJeP8rNvrRXAkf");

/// Ціни предметів у MagicToken
const ITEM_PRICES: [u64; 4] = [
    10, // Шабля козака
    15, // Посох старійшини
    20, // Броня характерника
    25, // Бойовий браслет
];

#[program]
pub mod marketplace {
    use super::*;

    /// Ініціалізує marketplace config
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.marketplace_config;
        config.admin = ctx.accounts.admin.key();
        config.magic_token_mint = ctx.accounts.magic_token_mint.key();
        config.bump = ctx.bumps.marketplace_config;
        msg!("Marketplace initialized");
        Ok(())
    }

    /// Продаж предмета: спалює NFT metadata + мінтить MagicToken продавцю
    pub fn sell_item<'a>(ctx: Context<'a, SellItem<'a>>, item_type: u8) -> Result<()> {
        require!(item_type < 4, MarketplaceError::InvalidItemType);

        // Перевірка що гравець є власником предмета
        let item = &ctx.accounts.item_metadata;
        require!(
            item.owner == ctx.accounts.seller.key(),
            MarketplaceError::NotOwner
        );
        require!(
            item.item_type == item_type,
            MarketplaceError::WrongItemType
        );

        let price = ITEM_PRICES[item_type as usize];

        // Спалюємо item metadata (закриваємо акаунт)
        let item_metadata = &mut ctx.accounts.item_metadata;
        item_metadata.owner = Pubkey::default();

        // Мінтимо MagicToken продавцю через PDA authority
        let bump = ctx.accounts.marketplace_config.bump;
        let seeds: &[&[u8]] = &[b"marketplace_config", &[bump]];
        let signer_seeds = &[seeds];

        token_2022::mint_to(
            CpiContext::new_with_signer(
                anchor_spl::token_2022::ID,
                MintTo {
                    mint: ctx.accounts.magic_token_mint.to_account_info(),
                    to: ctx.accounts.seller_magic_token_account.to_account_info(),
                    authority: ctx.accounts.marketplace_config.to_account_info(),
                },
                signer_seeds,
            ),
            price,
        )?;

        msg!("Sold item type={} for {} MagicTokens", item_type, price);
        Ok(())
    }
}

#[account]
pub struct MarketplaceConfig {
    pub admin: Pubkey,
    pub magic_token_mint: Pubkey,
    pub bump: u8,
}

impl MarketplaceConfig {
    pub const LEN: usize = 8 + 32 + 32 + 1;
}

#[account]
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = MarketplaceConfig::LEN,
        seeds = [b"marketplace_config"],
        bump
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,

    /// CHECK: magic token mint акаунт
    pub magic_token_mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SellItem<'info> {
    #[account(
        mut,
        seeds = [b"marketplace_config"],
        bump = marketplace_config.bump
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,

    #[account(mut)]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// CHECK: magic token mint акаунт для мінтингу
    #[account(mut)]
    pub magic_token_mint: AccountInfo<'info>,

    /// CHECK: seller MagicToken акаунт
    #[account(mut)]
    pub seller_magic_token_account: UncheckedAccount<'info>,

    pub seller: Signer<'info>,

    pub token_program: Program<'info, Token2022>,
}

#[error_code]
pub enum MarketplaceError {
    #[msg("Невірний тип предмета")]
    InvalidItemType,
    #[msg("Ти не є власником цього предмета")]
    NotOwner,
    #[msg("Тип предмета не співпадає")]
    WrongItemType,
}