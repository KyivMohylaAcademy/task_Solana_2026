use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use resource_manager::cpi::accounts::BurnResource;
use resource_manager::cpi::burn_resource;
use resource_manager::GameConfig;
use item_nft::cpi::accounts::CreateItemNft;
use item_nft::cpi::create_item_nft;
use item_nft::ItemCollection;

declare_id!("5BLa56P3DuD4GjLboQGKFAVv8gsYhP5A4eHLQHxS6QLR");

/// Crafting — програма крафту NFT-предметів з ресурсів.
///
/// Гравець може створити предмет якщо має достатню кількість ресурсів.
/// Ресурси спалюються, а NFT-предмет мінтиться гравцю.
///
/// Рецепти:
/// - Шабля козака (0): 3× Залізо + 1× Дерево + 1× Шкіра
/// - Посох старійшини (1): 2× Дерево + 1× Золото + 1× Алмаз
/// - Броня характерника (2): 4× Шкіра + 2× Залізо + 1× Золото
/// - Бойовий браслет (3): 4× Залізо + 2× Золото + 2× Алмаз
#[program]
pub mod crafting {
    use super::*;

    /// Крафт предмета з ресурсів.
    ///
    /// Перевіряє рецепт, спалює необхідні ресурси через CPI до resource_manager,
    /// створює NFT через CPI до item_nft.
    ///
    /// remaining_accounts очікує 6 акаунтів (3 пари mint+ATA за рецептом):
    /// - [0]: mint інгредієнта 0, [1]: ATA інгредієнта 0
    /// - [2]: mint інгредієнта 1, [3]: ATA інгредієнта 1
    /// - [4]: mint інгредієнта 2, [5]: ATA інгредієнта 2
    /// Порядок пар відповідає порядку інгредієнтів у рецепті.
    pub fn craft_item<'info>(
        ctx: Context<'_, '_, '_, 'info, CraftItem<'info>>,
        item_type: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        require!(item_type <= 3, CraftingError::InvalidItemType);
        require!(
            ctx.remaining_accounts.len() >= 6,
            CraftingError::InsufficientAccounts
        );

        // Отримуємо рецепт для даного типу предмета
        let recipe = get_recipe(item_type)?;

        msg!("Crafting item type: {}", item_type);

        // CPI authority seeds
        let cpi_bump = ctx.bumps.cpi_authority;
        let cpi_seeds: &[&[u8]] = &[b"cpi_authority", &[cpi_bump]];
        let signer_seeds = &[cpi_seeds];

        // Спалюємо кожен ресурс за рецептом через CPI до resource_manager
        for (i, (resource_id, amount)) in recipe.iter().enumerate() {
            let resource_mint = ctx.remaining_accounts[i * 2].clone();
            let player_token_account =
                ctx.remaining_accounts[i * 2 + 1].clone();

            msg!("Burning {} of resource {}", amount, resource_id);

            let burn_accounts = BurnResource {
                player: ctx.accounts.player.to_account_info(),
                caller_auth: ctx.accounts.cpi_authority.to_account_info(),
                game_config: ctx.accounts.game_config.to_account_info(),
                resource_mint: resource_mint,
                player_token_account: player_token_account,
                token_2022_program: ctx.accounts.token_2022_program.to_account_info(),
            };

            burn_resource(
                CpiContext::new_with_signer(
                    ctx.accounts.resource_manager_program.to_account_info(),
                    burn_accounts,
                    signer_seeds,
                ),
                *resource_id,
                *amount,
            )?;
        }

        msg!("Resources burned. Creating NFT...");

        // Створюємо NFT через CPI до item_nft
        let create_accounts = CreateItemNft {
            collection: ctx.accounts.item_collection.to_account_info(),
            payer: ctx.accounts.player.to_account_info(),
            player: ctx.accounts.player.to_account_info(),
            nft_authority: ctx.accounts.nft_authority.to_account_info(),
            nft_mint: ctx.accounts.nft_mint.to_account_info(),
            player_ata: ctx.accounts.nft_token_account.to_account_info(),
            metadata_account: ctx.accounts.metadata_account.to_account_info(),
            master_edition: ctx.accounts.master_edition.to_account_info(),
            item_metadata: ctx.accounts.item_metadata.to_account_info(),
            caller_auth: ctx.accounts.cpi_authority.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
            metadata_program: ctx.accounts.metadata_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        create_item_nft(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                create_accounts,
                signer_seeds,
            ),
            item_type,
            name,
            symbol,
            uri,
        )?;

        msg!("Item NFT created successfully!");
        Ok(())
    }
}

/// Повертає рецепт крафту: вектор пар (resource_id, кількість).
///
/// Resource IDs: WOOD=0, IRON=1, GOLD=2, LEATHER=3, STONE=4, DIAMOND=5
fn get_recipe(item_type: u8) -> Result<Vec<(u8, u64)>> {
    match item_type {
        0 => Ok(vec![(1, 3), (0, 1), (3, 1)]),     // Шабля: 3 Iron + 1 Wood + 1 Leather
        1 => Ok(vec![(0, 2), (2, 1), (5, 1)]),     // Посох: 2 Wood + 1 Gold + 1 Diamond
        2 => Ok(vec![(3, 4), (1, 2), (2, 1)]),     // Броня: 4 Leather + 2 Iron + 1 Gold
        3 => Ok(vec![(1, 4), (2, 2), (5, 2)]),     // Браслет: 4 Iron + 2 Gold + 2 Diamond
        _ => Err(CraftingError::InvalidItemType.into()),
    }
}

/// Акаунти для крафту предмета.
#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CraftItem<'info> {
    /// Гравець — підписант та платник
    #[account(mut)]
    pub player: Signer<'info>,

    // === Resource Manager accounts ===

    /// Конфігурація гри (GameConfig PDA з resource_manager)
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager::ID,
    )]
    pub game_config: Account<'info, GameConfig>,

    // === Item NFT accounts ===

    /// Колекція предметів (ItemCollection PDA з item_nft)
    #[account(
        mut,
        seeds = [b"item_collection"],
        bump = item_collection.bump,
        seeds::program = item_nft::ID,
    )]
    pub item_collection: Account<'info, ItemCollection>,

    /// NFT authority PDA від item_nft
    /// CHECK: PDA item_nft з seeds [b"nft_authority"]
    #[account(
        seeds = [b"nft_authority"],
        bump = item_collection.nft_authority_bump,
        seeds::program = item_nft::ID,
    )]
    pub nft_authority: UncheckedAccount<'info>,

    /// Новий мінт для NFT (keypair, Signer)
    #[account(mut)]
    pub nft_mint: Signer<'info>,

    /// Token account гравця для NFT
    /// CHECK: Створюється через CPI в item_nft
    #[account(mut)]
    pub nft_token_account: UncheckedAccount<'info>,

    /// Metaplex Metadata PDA
    /// CHECK: seeds = ["metadata", metadata_program_id, nft_mint]
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// Metaplex Master Edition PDA
    /// CHECK: seeds = ["metadata", metadata_program_id, nft_mint, "edition"]
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// ItemMetadata PDA (створюється item_nft)
    /// CHECK: PDA item_nft з seeds [b"item_metadata", nft_mint.key()]
    #[account(mut)]
    pub item_metadata: UncheckedAccount<'info>,

    // === CPI Authority ===

    /// CPI authority PDA від crafting програми
    /// CHECK: PDA цієї програми з seeds [b"cpi_authority"]
    #[account(
        seeds = [b"cpi_authority"],
        bump,
    )]
    pub cpi_authority: UncheckedAccount<'info>,

    // === Programs ===

    /// Resource manager програма
    /// CHECK: Перевіряється за program ID
    pub resource_manager_program: UncheckedAccount<'info>,

    /// Item NFT програма
    /// CHECK: Перевіряється за program ID
    pub item_nft_program: UncheckedAccount<'info>,

    /// Token-2022 програма (для спалення ресурсів)
    /// CHECK: SPL Token-2022 program
    pub token_2022_program: UncheckedAccount<'info>,

    /// Token програма (для NFT)
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

/// Коди помилок для програми crafting.
#[error_code]
pub enum CraftingError {
    /// Невірний тип предмета (має бути 0-3)
    #[msg("Invalid item type (must be 0-3)")]
    InvalidItemType,
    /// Недостатньо remaining accounts
    #[msg("Insufficient remaining accounts (need 6: 3 mint+ATA pairs for recipe)")]
    InsufficientAccounts,
}
