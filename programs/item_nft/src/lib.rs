use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Burn, CloseAccount, Mint, MintTo, Token, TokenAccount};
use mpl_token_metadata::instructions::{
    CreateMasterEditionV3CpiBuilder, CreateMetadataAccountV3CpiBuilder,
};
use mpl_token_metadata::types::{Creator, DataV2};

declare_id!("9uYdFs7H7iZjRYMB2r3kvGATe5ZUaKvbZrkTijR5sGEw");

/// Item NFT — керування Metaplex NFT-предметами для крафту в грі "Козацький бізнес".
///
/// Предмети створюються через програму crafting та спалюються через marketplace.
/// Кожен предмет — це унікальний NFT зі стандартом Metaplex Token Metadata.
///
/// Типи предметів:
/// - 0: Шабля козака
/// - 1: Посох старійшини
/// - 2: Броня характерника
/// - 3: Бойовий браслет
#[program]
pub mod item_nft {
    use super::*;

    /// Ініціалізація колекції предметів.
    ///
    /// Створює ItemCollection PDA та nft_authority PDA.
    /// nft_authority буде mint/update authority для всіх NFT.
    pub fn initialize_collection(
        ctx: Context<InitializeCollection>,
        crafting_program: Pubkey,
        marketplace_program: Pubkey,
    ) -> Result<()> {
        let collection = &mut ctx.accounts.collection;
        collection.admin = ctx.accounts.admin.key();
        collection.crafting_program = crafting_program;
        collection.marketplace_program = marketplace_program;
        collection.item_count = 0;
        collection.nft_authority_bump = ctx.bumps.nft_authority;
        collection.bump = ctx.bumps.collection;

        msg!("ItemCollection initialized");
        Ok(())
    }

    /// Створення нового NFT-предмета — CPI-only від crafting програми.
    ///
    /// Створює SPL Token мінт (decimals=0, supply=1), ATA, Metaplex Metadata
    /// та Master Edition. Зберігає ItemMetadata PDA.
    pub fn create_item_nft(
        ctx: Context<CreateItemNft>,
        item_type: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        require!(item_type <= 3, ItemNftError::InvalidItemType);

        let collection = &mut ctx.accounts.collection;

        // Перевірка авторизації — caller_auth має бути PDA crafting програми
        let expected_caller = Pubkey::find_program_address(
            &[b"cpi_authority"],
            &collection.crafting_program,
        )
        .0;
        require_eq!(
            ctx.accounts.caller_auth.key(),
            expected_caller,
            ItemNftError::UnauthorizedCaller
        );

        let nft_authority_bump = collection.nft_authority_bump;
        let nft_authority_seeds: &[&[u8]] = &[b"nft_authority", &[nft_authority_bump]];

        // 1. Створюємо акаунт для NFT мінта
        let mint_rent = Rent::get()?.minimum_balance(Mint::LEN);
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::create_account(
                &ctx.accounts.payer.key(),
                &ctx.accounts.nft_mint.key(),
                mint_rent,
                Mint::LEN as u64,
                &token::ID,
            ),
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // 2. Ініціалізуємо мінт (decimals=0, authority=nft_authority PDA)
        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::InitializeMint {
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            0,
            &ctx.accounts.nft_authority.key(),
            Some(&ctx.accounts.nft_authority.key()),
        )?;

        // 3. Створюємо ATA для гравця
        anchor_spl::associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.payer.to_account_info(),
                associated_token: ctx.accounts.player_ata.to_account_info(),
                authority: ctx.accounts.player.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        ))?;

        // 4. Мінтимо 1 NFT на ATA гравця
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    to: ctx.accounts.player_ata.to_account_info(),
                    authority: ctx.accounts.nft_authority.to_account_info(),
                },
                &[nft_authority_seeds],
            ),
            1,
        )?;

        // 5. Створюємо Metaplex Token Metadata
        let creators = vec![Creator {
            address: ctx.accounts.nft_authority.key(),
            verified: true,
            share: 100,
        }];

        CreateMetadataAccountV3CpiBuilder::new(&ctx.accounts.metadata_program)
            .metadata(&ctx.accounts.metadata_account)
            .mint(&ctx.accounts.nft_mint)
            .mint_authority(&ctx.accounts.nft_authority)
            .payer(&ctx.accounts.payer)
            .update_authority(&ctx.accounts.nft_authority, true)
            .system_program(&ctx.accounts.system_program)
            .data(DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points: 0,
                creators: Some(creators),
                collection: None,
                uses: None,
            })
            .is_mutable(true)
            .invoke_signed(&[nft_authority_seeds])?;

        // 6. Створюємо Master Edition (робить токен справжнім NFT)
        CreateMasterEditionV3CpiBuilder::new(&ctx.accounts.metadata_program)
            .edition(&ctx.accounts.master_edition)
            .mint(&ctx.accounts.nft_mint)
            .update_authority(&ctx.accounts.nft_authority)
            .mint_authority(&ctx.accounts.nft_authority)
            .payer(&ctx.accounts.payer)
            .metadata(&ctx.accounts.metadata_account)
            .token_program(&ctx.accounts.token_program)
            .system_program(&ctx.accounts.system_program)
            .max_supply(0)
            .invoke_signed(&[nft_authority_seeds])?;

        // 7. Заповнюємо ItemMetadata PDA
        let item_metadata = &mut ctx.accounts.item_metadata;
        item_metadata.item_type = item_type;
        item_metadata.owner = ctx.accounts.player.key();
        item_metadata.mint = ctx.accounts.nft_mint.key();
        item_metadata.bump = ctx.bumps.item_metadata;

        // Збільшуємо лічильник предметів
        collection.item_count += 1;

        msg!(
            "Created item NFT type {} mint: {}",
            item_type,
            ctx.accounts.nft_mint.key()
        );
        Ok(())
    }

    /// Спалення NFT-предмета — CPI-only від marketplace.
    ///
    /// Спалює NFT, закриває token account та ItemMetadata PDA.
    /// Рент повертається власнику.
    pub fn burn_item_nft(ctx: Context<BurnItemNft>) -> Result<()> {
        let collection = &ctx.accounts.collection;

        // Перевірка авторизації — caller_auth має бути PDA маркетплейсу
        let expected_caller = Pubkey::find_program_address(
            &[b"cpi_authority"],
            &collection.marketplace_program,
        )
        .0;
        require_eq!(
            ctx.accounts.caller_auth.key(),
            expected_caller,
            ItemNftError::UnauthorizedCaller
        );

        // Спалюємо NFT
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    from: ctx.accounts.nft_token_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            1,
        )?;

        // Закриваємо token account
        token::close_account(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            CloseAccount {
                account: ctx.accounts.nft_token_account.to_account_info(),
                destination: ctx.accounts.owner.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ))?;

        msg!(
            "Burned item NFT: {}",
            ctx.accounts.nft_mint.key()
        );
        Ok(())
    }
}

/// Конфігурація колекції предметів.
///
/// PDA seeds: [b"item_collection"]
#[account]
pub struct ItemCollection {
    /// Адміністратор
    pub admin: Pubkey,
    /// Crafting program ID — може створювати NFT
    pub crafting_program: Pubkey,
    /// Marketplace program ID — може спалювати NFT
    pub marketplace_program: Pubkey,
    /// Загальна кількість створених предметів
    pub item_count: u64,
    /// Bump для nft_authority PDA
    pub nft_authority_bump: u8,
    /// Bump для collection PDA
    pub bump: u8,
}

/// Метадані NFT-предмета.
///
/// PDA seeds: [b"item_metadata", nft_mint.key()]
#[account]
pub struct ItemMetadata {
    /// Тип предмета (0=Шабля, 1=Посох, 2=Броня, 3=Браслет)
    pub item_type: u8,
    /// Власник предмета
    pub owner: Pubkey,
    /// Мінт NFT
    pub mint: Pubkey,
    /// Bump для PDA
    pub bump: u8,
}

/// Акаунти для ініціалізації колекції.
#[derive(Accounts)]
pub struct InitializeCollection<'info> {
    /// Адміністратор (підписант та платник)
    #[account(mut)]
    pub admin: Signer<'info>,

    /// ItemCollection PDA
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 32 + 32 + 8 + 1 + 1,
        seeds = [b"item_collection"],
        bump,
    )]
    pub collection: Account<'info, ItemCollection>,

    /// NFT authority PDA
    /// CHECK: PDA для mint/update authority NFT
    #[account(
        seeds = [b"nft_authority"],
        bump,
    )]
    pub nft_authority: UncheckedAccount<'info>,

    /// Системна програма
    pub system_program: Program<'info, System>,
}

/// Акаунти для створення NFT-предмета.
#[derive(Accounts)]
#[instruction(item_type: u8, name: String, symbol: String, uri: String)]
pub struct CreateItemNft<'info> {
    /// ItemCollection (mut для збільшення item_count)
    #[account(
        mut,
        seeds = [b"item_collection"],
        bump = collection.bump,
    )]
    pub collection: Account<'info, ItemCollection>,

    /// Платник за створення акаунтів
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Гравець (отримувач NFT) — має підписати
    pub player: Signer<'info>,

    /// NFT authority PDA
    /// CHECK: PDA для підпису мінт операцій
    #[account(
        seeds = [b"nft_authority"],
        bump = collection.nft_authority_bump,
    )]
    pub nft_authority: UncheckedAccount<'info>,

    /// Новий NFT мінт (keypair, Signer)
    #[account(mut)]
    pub nft_mint: Signer<'info>,

    /// ATA гравця для NFT
    /// CHECK: Створюється в цій інструкції через CPI
    #[account(mut)]
    pub player_ata: UncheckedAccount<'info>,

    /// Metaplex Metadata PDA
    /// CHECK: seeds = ["metadata", metadata_program_id, nft_mint]
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// Metaplex Master Edition PDA
    /// CHECK: seeds = ["metadata", metadata_program_id, nft_mint, "edition"]
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// ItemMetadata PDA
    #[account(
        init,
        payer = payer,
        space = 8 + 1 + 32 + 32 + 1,
        seeds = [b"item_metadata", nft_mint.key().as_ref()],
        bump,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// CPI caller authority від crafting програми (має бути Signer)
    pub caller_auth: Signer<'info>,

    /// Token програма
    pub token_program: Program<'info, Token>,

    /// Associated Token програма
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Metaplex Token Metadata програма
    /// CHECK: Metaplex program ID
    #[account(address = mpl_token_metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,

    /// Системна програма
    pub system_program: Program<'info, System>,

    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
}

/// Акаунти для спалення NFT-предмета.
#[derive(Accounts)]
pub struct BurnItemNft<'info> {
    /// ItemCollection
    #[account(
        seeds = [b"item_collection"],
        bump = collection.bump,
    )]
    pub collection: Account<'info, ItemCollection>,

    /// Власник NFT (має підписати)
    #[account(mut)]
    pub owner: Signer<'info>,

    /// NFT мінт
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,

    /// Token account власника для NFT
    #[account(
        mut,
        constraint = nft_token_account.owner == owner.key(),
        constraint = nft_token_account.mint == nft_mint.key(),
        constraint = nft_token_account.amount == 1,
    )]
    pub nft_token_account: Account<'info, TokenAccount>,

    /// ItemMetadata PDA (закривається, рент -> owner)
    #[account(
        mut,
        seeds = [b"item_metadata", nft_mint.key().as_ref()],
        bump = item_metadata.bump,
        close = owner,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// CPI caller authority від marketplace (має бути Signer)
    pub caller_auth: Signer<'info>,

    /// Token програма
    pub token_program: Program<'info, Token>,
}

/// Коди помилок для item_nft.
#[error_code]
pub enum ItemNftError {
    /// Невірний тип предмета (має бути 0-3)
    #[msg("Invalid item type (must be 0-3)")]
    InvalidItemType,
    /// Неавторизований виклик
    #[msg("Unauthorized caller")]
    UnauthorizedCaller,
    /// Тільки адміністратор може виконати цю дію
    #[msg("Only admin can perform this action")]
    OnlyAdmin,
}
