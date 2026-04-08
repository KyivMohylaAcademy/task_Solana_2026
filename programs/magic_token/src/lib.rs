use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::extension::metadata_pointer::instruction::initialize as init_metadata_pointer;
use spl_token_2022::instruction::initialize_mint2;
use spl_token_metadata_interface::instruction::initialize as init_token_metadata;
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;
use std::mem::size_of;

declare_id!("CLennJDsGwnsVFAGfju1yH8frdb3QgC8i7mdXFRxhPKx");

/// MagicToken — SPL Token-2022 ігрова валюта, мінтиться тільки через Marketplace.
///
/// Токени MagicToken можна отримати лише через продаж предметів у програмі Marketplace.
/// Прямий мінтинг через Token Program — заборонено.
#[program]
pub mod magic_token {
    use super::*;

    /// Ініціалізація MagicToken з Token-2022 та MetadataPointer розширенням.
    ///
    /// Створює MagicTokenConfig PDA, Token-2022 мінт з он-чейн метаданими.
    /// Мінт authority — PDA з seeds [b"magic_mint_authority"].
    pub fn initialize_magic_token(
        ctx: Context<InitializeMagicToken>,
        marketplace_program: Pubkey,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.mint = ctx.accounts.magic_mint.key();
        config.marketplace_program = marketplace_program;
        config.mint_authority_bump = ctx.bumps.mint_authority;
        config.bump = ctx.bumps.config;

        let mint_authority_key = ctx.accounts.mint_authority.key();
        let magic_mint_key = ctx.accounts.magic_mint.key();

        // Розрахунок розміру для Token-2022 мінт з MetadataPointer
        let token_metadata = TokenMetadata {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            ..Default::default()
        };

        let extension_len = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(
            &[ExtensionType::MetadataPointer],
        )
        .map_err(|_| MagicTokenError::OnlyAdmin)?;

        let metadata_len = token_metadata
            .get_packed_len()
            .map_err(|_| MagicTokenError::OnlyAdmin)?;
        let metadata_tlv_len = metadata_len
            + size_of::<ExtensionType>()
            + size_of::<spl_token_2022::extension::Length>();
        let total_len = extension_len + metadata_tlv_len;
        let lamports = Rent::get()?.minimum_balance(total_len);

        // 1. Створюємо акаунт для мінт
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::create_account(
                &ctx.accounts.admin.key(),
                &magic_mint_key,
                lamports,
                extension_len as u64,
                &spl_token_2022::id(),
            ),
            &[
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.magic_mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // 2. Ініціалізуємо MetadataPointer розширення
        anchor_lang::solana_program::program::invoke(
            &init_metadata_pointer(
                &spl_token_2022::id(),
                &magic_mint_key,
                Some(mint_authority_key),
                Some(magic_mint_key),
            )?,
            &[
                ctx.accounts.magic_mint.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;

        // 3. Ініціалізуємо мінт (decimals = 0)
        anchor_lang::solana_program::program::invoke(
            &initialize_mint2(
                &spl_token_2022::id(),
                &magic_mint_key,
                &mint_authority_key,
                Some(&mint_authority_key),
                0,
            )?,
            &[
                ctx.accounts.magic_mint.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;

        // 4. Ініціалізуємо метадані на мінт-акаунті
        let mint_authority_bump_seed = [ctx.bumps.mint_authority];
        let mint_authority_seeds: &[&[u8]] = &[b"magic_mint_authority", &mint_authority_bump_seed];
        let signer_seeds = [mint_authority_seeds];

        anchor_lang::solana_program::program::invoke_signed(
            &init_token_metadata(
                &spl_token_2022::id(),
                &magic_mint_key,
                &mint_authority_key,
                &magic_mint_key,
                &mint_authority_key,
                name,
                symbol,
                uri,
            ),
            &[
                ctx.accounts.magic_mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
            &signer_seeds,
        )?;

        msg!("MagicToken initialized: {}", magic_mint_key);
        Ok(())
    }

    /// Мінтинг MagicTokens — CPI-only інструкція від marketplace.
    ///
    /// Перевіряє що caller_auth є PDA маркетплейс програми,
    /// потім мінтить вказану кількість токенів на акаунт отримувача.
    pub fn mint_magic_tokens(ctx: Context<MintMagicTokens>, amount: u64) -> Result<()> {
        let config = &ctx.accounts.config;

        // Перевірка авторизації — caller_auth має бути PDA маркетплейсу
        let expected_caller = Pubkey::find_program_address(
            &[b"cpi_authority"],
            &config.marketplace_program,
        )
        .0;

        require_eq!(
            ctx.accounts.caller_auth.key(),
            expected_caller,
            MagicTokenError::UnauthorizedCaller
        );

        // Мінтимо токени через mint_authority PDA
        let mint_authority_bump_seed = [config.mint_authority_bump];
        let mint_authority_seeds: &[&[u8]] = &[b"magic_mint_authority", &mint_authority_bump_seed];
        let signer_seeds = [mint_authority_seeds];

        token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token_2022::MintTo {
                    mint: ctx.accounts.magic_mint.to_account_info(),
                    to: ctx.accounts.recipient_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &signer_seeds,
            ),
            amount,
        )?;

        msg!("Minted {} MagicTokens", amount);
        Ok(())
    }
}

/// Конфігурація MagicToken.
///
/// PDA seeds: [b"magic_config"]
#[account]
pub struct MagicTokenConfig {
    /// Адміністратор
    pub admin: Pubkey,
    /// SPL Token-2022 мінт
    pub mint: Pubkey,
    /// Marketplace program ID — тільки ця програма може мінтити
    pub marketplace_program: Pubkey,
    /// Bump для mint_authority PDA
    pub mint_authority_bump: u8,
    /// Bump для config PDA
    pub bump: u8,
}

/// Акаунти для ініціалізації MagicToken.
#[derive(Accounts)]
pub struct InitializeMagicToken<'info> {
    /// Адміністратор (підписант та платник)
    #[account(mut)]
    pub admin: Signer<'info>,

    /// MagicTokenConfig PDA
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 32 + 32 + 1 + 1,
        seeds = [b"magic_config"],
        bump,
    )]
    pub config: Account<'info, MagicTokenConfig>,

    /// Новий Token-2022 мінт (keypair, Signer)
    #[account(mut)]
    pub magic_mint: Signer<'info>,

    /// Mint authority PDA (seeds = ["magic_mint_authority"])
    /// CHECK: PDA для підпису мінт операцій
    #[account(
        seeds = [b"magic_mint_authority"],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// Token-2022 програма
    pub token_program: Program<'info, token_2022::Token2022>,

    /// Системна програма
    pub system_program: Program<'info, System>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}

/// Акаунти для мінтингу MagicTokens (CPI від marketplace).
#[derive(Accounts)]
pub struct MintMagicTokens<'info> {
    /// MagicTokenConfig
    #[account(
        seeds = [b"magic_config"],
        bump = config.bump,
    )]
    pub config: Account<'info, MagicTokenConfig>,

    /// MagicToken мінт
    /// CHECK: Перевіряється через config.mint
    #[account(
        mut,
        constraint = magic_mint.key() == config.mint @ MagicTokenError::UnauthorizedCaller,
    )]
    pub magic_mint: UncheckedAccount<'info>,

    /// Token account отримувача
    /// CHECK: Валідується token програмою під час CPI
    #[account(mut)]
    pub recipient_token_account: UncheckedAccount<'info>,

    /// Mint authority PDA
    /// CHECK: PDA з seeds [b"magic_mint_authority"]
    #[account(
        seeds = [b"magic_mint_authority"],
        bump = config.mint_authority_bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// CPI caller authority від marketplace (має бути підписантом)
    pub caller_auth: Signer<'info>,

    /// Token-2022 програма
    pub token_program: Program<'info, token_2022::Token2022>,
}

/// Коди помилок для magic_token.
#[error_code]
pub enum MagicTokenError {
    /// Неавторизований виклик — тільки marketplace може мінтити
    #[msg("Unauthorized caller - only marketplace can mint")]
    UnauthorizedCaller,
    /// Тільки адміністратор може виконати цю дію
    #[msg("Only admin can perform this action")]
    OnlyAdmin,
}
