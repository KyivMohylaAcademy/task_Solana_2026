use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::token_2022::{self, InitializeMint2, Token2022};
use anchor_spl::token_2022_extensions::{
    metadata_pointer_initialize, token_metadata_initialize,
    MetadataPointerInitialize, TokenMetadataInitialize,
};

use crate::state::*;
use crate::error::ResourceManagerError;

/// Розмір базового мінт-акаунту з MetadataPointer + буфер для TokenMetadata.
///
/// 170 байт — base Mint (82) + AccountType (1) + MetadataPointer TLV (68) + вирівнювання.
/// Додатково 300 байт для токен-метаданих (name, symbol, uri + заголовки).
const RESOURCE_MINT_SPACE: usize = 470;

/// Створює новий SPL Token-2022 мінт для ресурсу з розширеннями MetadataPointer та TokenMetadata.
///
/// Mint authority = PDA seeds=["mint_authority"].
/// Decimals = 0 (ресурси — цілі одиниці).
/// MetadataPointer вказує на сам мінт (вбудовані метадані).
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct CreateResourceMint<'info> {
    /// Конфігурація гри для оновлення адреси мінту
    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump,
        has_one = admin @ ResourceManagerError::UnauthorizedAdmin,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// PDA-авторитет мінту
    #[account(
        seeds = [b"mint_authority"],
        bump = mint_authority.bump,
    )]
    pub mint_authority: Account<'info, MintAuthority>,

    /// Адміністратор — платник та підписант
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Новий мінт-акаунт (keypair генерується клієнтом, передається як signer)
    /// CHECK: Ініціалізується в цій інструкції через anchor-spl CPI
    #[account(mut, signer)]
    pub mint: AccountInfo<'info>,

    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateResourceMint>,
    resource_id: u8,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    require!(resource_id < 6, ResourceManagerError::InvalidResourceId);
    require!(
        ctx.accounts.game_config.resource_mints[resource_id as usize] == Pubkey::default(),
        ResourceManagerError::MintAlreadySet
    );

    let mint_key = ctx.accounts.mint.key();
    let mint_auth_key = ctx.accounts.mint_authority.key();
    let mint_authority_bump = ctx.accounts.mint_authority.bump;
    let mint_authority_seeds: &[&[u8]] = &[b"mint_authority", &[mint_authority_bump]];

    let total_space = RESOURCE_MINT_SPACE;
    let lamports = ctx.accounts.rent.minimum_balance(total_space);

    // 1. Створюємо акаунт через SystemProgram
    invoke(
        &anchor_lang::solana_program::system_instruction::create_account(
            ctx.accounts.admin.key,
            &mint_key,
            lamports,
            total_space as u64,
            &anchor_spl::token_2022::ID,
        ),
        &[
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // 2. Ініціалізуємо MetadataPointer (вказує на сам мінт — вбудовані метадані)
    metadata_pointer_initialize(
        CpiContext::new(
            anchor_spl::token_2022::ID,
            MetadataPointerInitialize {
                token_program_id: ctx.accounts.token_2022_program.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(mint_auth_key),
        Some(mint_key),
    )?;

    // 3. Ініціалізуємо мінт (decimals=0, mint authority=PDA)
    token_2022::initialize_mint2(
        CpiContext::new(
            anchor_spl::token_2022::ID,
            InitializeMint2 {
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        0,
        &mint_auth_key,
        None,
    )?;

    // 4. Ініціалізуємо TokenMetadata (підписує mint_authority PDA)
    token_metadata_initialize(
        CpiContext::new_with_signer(
            anchor_spl::token_2022::ID,
            TokenMetadataInitialize {
                program_id: ctx.accounts.token_2022_program.to_account_info(),
                metadata: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.mint_authority.to_account_info(),
                mint_authority: ctx.accounts.mint_authority.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
            &[mint_authority_seeds],
        ),
        name.clone(),
        symbol.clone(),
        uri.clone(),
    )?;

    // 5. Зберігаємо адресу мінту в конфігурації гри
    ctx.accounts.game_config.resource_mints[resource_id as usize] = mint_key;

    msg!(
        "Мінт ресурсу {} ({}) створено: {}",
        resource_id,
        symbol,
        mint_key
    );
    Ok(())
}
