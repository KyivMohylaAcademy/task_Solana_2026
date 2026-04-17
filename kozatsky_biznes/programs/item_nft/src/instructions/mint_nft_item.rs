use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{self, InitializeMint2, MintTo, Token2022},
    token_2022_extensions::{
        metadata_pointer_initialize, token_metadata_initialize,
        MetadataPointerInitialize, TokenMetadataInitialize,
    },
};

use crate::state::*;
use crate::error::ItemNftError;

/// Розмір мінт-акаунту з MetadataPointer + буфер для TokenMetadata.
///
/// 170 байт — base Mint (82) + AccountType (1) + MetadataPointer TLV (68) + вирівнювання.
/// Додатково 300 байт для токен-метаданих (name, symbol, uri + заголовки).
const NFT_MINT_SPACE: usize = 470;

/// Мінтить NFT предмет (Token-2022, supply=1) на акаунт гравця.
///
/// Безпека: викликається виключно через CPI з програми crafting.
/// Створює мінт з MetadataPointer + TokenMetadata та ItemMetadata PDA.
#[derive(Accounts)]
#[instruction(item_type: u8, item_name: String, item_symbol: String, item_uri: String)]
pub struct MintNftItem<'info> {
    /// Конфігурація ItemNft
    #[account(
        seeds = [b"item_nft_config"],
        bump = item_nft_config.bump,
    )]
    pub item_nft_config: Account<'info, ItemNftConfig>,

    /// PDA-авторитет мінту
    #[account(
        seeds = [b"nft_mint_authority"],
        bump = nft_mint_authority.bump,
    )]
    pub nft_mint_authority: Account<'info, NftMintAuthority>,

    /// Авторизований виклик (crafting authority PDA)
    /// CHECK: Перевіряється в коді: має бути signer та відповідати crafting_authority
    #[account(
        signer,
        constraint = caller_authority.key() == item_nft_config.crafting_authority
            @ ItemNftError::UnauthorizedMintCaller
    )]
    pub caller_authority: AccountInfo<'info>,

    /// Платник (зазвичай гравець)
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Власник (гравець), що отримує NFT
    /// CHECK: Просто адреса, токен-акаунт прив'язано до неї
    pub player: AccountInfo<'info>,

    /// Новий NFT мінт-акаунт (keypair від клієнта)
    /// CHECK: Ініціалізується в цій інструкції через anchor-spl CPI
    #[account(mut, signer)]
    pub nft_mint: AccountInfo<'info>,

    /// Метадані предмету (PDA)
    #[account(
        init,
        payer = payer,
        space = ItemMetadata::SPACE,
        seeds = [b"item_metadata", nft_mint.key().as_ref()],
        bump,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// Асоційований токен-акаунт гравця для NFT
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = nft_mint,
        associated_token::authority = player,
        associated_token::token_program = token_2022_program,
    )]
    pub player_nft_token_account: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<MintNftItem>,
    item_type: u8,
    item_name: String,
    item_symbol: String,
    item_uri: String,
) -> Result<()> {
    require!(item_type < 4, ItemNftError::InvalidItemType);

    let mint_key = ctx.accounts.nft_mint.key();
    let mint_auth_key = ctx.accounts.nft_mint_authority.key();
    let mint_authority_bump = ctx.accounts.nft_mint_authority.bump;
    let mint_authority_seeds: &[&[u8]] = &[b"nft_mint_authority", &[mint_authority_bump]];

    let total_space = NFT_MINT_SPACE;
    let lamports = ctx.accounts.rent.minimum_balance(total_space);

    // 1. Створюємо акаунт для NFT мінту через SystemProgram
    invoke(
        &anchor_lang::solana_program::system_instruction::create_account(
            ctx.accounts.payer.key,
            &mint_key,
            lamports,
            total_space as u64,
            &anchor_spl::token_2022::ID,
        ),
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.nft_mint.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // 2. Ініціалізуємо MetadataPointer (вказує на сам мінт — вбудовані метадані)
    metadata_pointer_initialize(
        CpiContext::new(
            anchor_spl::token_2022::ID,
            MetadataPointerInitialize {
                token_program_id: ctx.accounts.token_2022_program.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
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
                mint: ctx.accounts.nft_mint.to_account_info(),
            },
        ),
        0,
        &mint_auth_key,
        None,
    )?;

    // 4. Ініціалізуємо TokenMetadata (підписує nft_mint_authority PDA)
    token_metadata_initialize(
        CpiContext::new_with_signer(
            anchor_spl::token_2022::ID,
            TokenMetadataInitialize {
                program_id: ctx.accounts.token_2022_program.to_account_info(),
                metadata: ctx.accounts.nft_mint.to_account_info(),
                update_authority: ctx.accounts.nft_mint_authority.to_account_info(),
                mint_authority: ctx.accounts.nft_mint_authority.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
            },
            &[mint_authority_seeds],
        ),
        item_name.clone(),
        item_symbol.clone(),
        item_uri.clone(),
    )?;

    // 5. Мінтимо 1 токен (NFT supply=1) на акаунт гравця
    token_2022::mint_to(
        CpiContext::new_with_signer(
            anchor_spl::token_2022::ID,
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.player_nft_token_account.to_account_info(),
                authority: ctx.accounts.nft_mint_authority.to_account_info(),
            },
            &[mint_authority_seeds],
        ),
        1,
    )?;

    // 6. Зберігаємо метадані предмету
    let item_metadata = &mut ctx.accounts.item_metadata;
    item_metadata.item_type = item_type;
    item_metadata.owner = ctx.accounts.player.key();
    item_metadata.mint = mint_key;
    item_metadata.bump = ctx.bumps.item_metadata;

    msg!(
        "Мінт NFT предмету типу {} ({}): {} для гравця {}",
        item_type,
        item_name,
        mint_key,
        ctx.accounts.player.key()
    );
    Ok(())
}
