use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
};
use resource_manager::{
    self,
    cpi::accounts::BurnResource,
    program::ResourceManager,
    state::{GameConfig, MintAuthority as RmMintAuthority},
};
use item_nft::{
    self,
    cpi::accounts::MintNftItem,
    program::ItemNft,
    state::{ItemNftConfig, NftMintAuthority},
};

use crate::state::*;
use crate::error::CraftingError;
use crate::constants::{RECIPES, ITEM_COUNT};

/// Крафтить предмет, спалюючи ресурси та мінтячи NFT.
///
/// Рецепти (resource_id, amount):
/// - Item 0 (Шабля):  [(1,3), (0,1), (3,1)]
/// - Item 1 (Посох):  [(0,2), (2,1), (5,1)]
/// - Item 2 (Броня):  [(3,4), (1,2), (2,1)]
/// - Item 3 (Браслет):[(1,4), (2,2), (5,2)]
///
/// Для кожного інгредієнту виконується BurnResource CPI до resource_manager.
/// Після спалення виконується MintNftItem CPI до item_nft.
#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CraftItem<'info> {
    /// Конфігурація гри (resource_manager) — для перевірки мінтів
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub game_config: Account<'info, GameConfig>,

    /// PDA-авторитет мінту resource_manager (потрібний для BurnResource accounts struct)
    #[account(
        seeds = [b"mint_authority"],
        bump = rm_mint_authority.bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub rm_mint_authority: Account<'info, RmMintAuthority>,

    /// CraftingAuthority PDA (підписує CPI до resource_manager та item_nft)
    #[account(
        seeds = [b"crafting_authority"],
        bump = crafting_authority.bump,
    )]
    pub crafting_authority: Account<'info, CraftingAuthority>,

    /// Конфігурація ItemNft
    #[account(
        seeds = [b"item_nft_config"],
        bump = item_nft_config.bump,
        seeds::program = item_nft_program.key(),
    )]
    pub item_nft_config: Account<'info, ItemNftConfig>,

    /// PDA-авторитет мінту item_nft
    #[account(
        seeds = [b"nft_mint_authority"],
        bump = nft_mint_authority.bump,
        seeds::program = item_nft_program.key(),
    )]
    pub nft_mint_authority: Account<'info, NftMintAuthority>,

    /// Гравець (підписант, власник ресурсів)
    #[account(mut)]
    pub player: Signer<'info>,

    // ---- Ресурсні мінти (3 інгредієнти) ----
    /// Мінт першого інгредієнту
    /// CHECK: Перевіряється в resource_manager через game_config
    #[account(mut)]
    pub resource_mint_0: AccountInfo<'info>,

    /// Мінт другого інгредієнту
    /// CHECK: Перевіряється в resource_manager через game_config
    #[account(mut)]
    pub resource_mint_1: AccountInfo<'info>,

    /// Мінт третього інгредієнту
    /// CHECK: Перевіряється в resource_manager через game_config
    #[account(mut)]
    pub resource_mint_2: AccountInfo<'info>,

    // ---- Токен-акаунти гравця для ресурсів ----
    /// Токен-акаунт гравця для першого інгредієнту
    #[account(
        mut,
        associated_token::mint = resource_mint_0,
        associated_token::authority = player,
        associated_token::token_program = token_2022_program,
    )]
    pub player_resource_account_0: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    /// Токен-акаунт гравця для другого інгредієнту
    #[account(
        mut,
        associated_token::mint = resource_mint_1,
        associated_token::authority = player,
        associated_token::token_program = token_2022_program,
    )]
    pub player_resource_account_1: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    /// Токен-акаунт гравця для третього інгредієнту
    #[account(
        mut,
        associated_token::mint = resource_mint_2,
        associated_token::authority = player,
        associated_token::token_program = token_2022_program,
    )]
    pub player_resource_account_2: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    // ---- NFT мінт та акаунт ----
    /// Новий NFT мінт-акаунт (keypair від клієнта)
    /// CHECK: Ініціалізується в item_nft::mint_nft_item
    #[account(mut, signer)]
    pub nft_mint: AccountInfo<'info>,

    /// Асоційований токен-акаунт гравця для NFT
    #[account(
        init_if_needed,
        payer = player,
        associated_token::mint = nft_mint,
        associated_token::authority = player,
        associated_token::token_program = token_2022_program,
    )]
    pub player_nft_token_account: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    /// ItemMetadata PDA (буде ініціалізовано в item_nft)
    /// CHECK: Перевіряється та ініціалізується в item_nft::mint_nft_item
    #[account(mut)]
    pub item_metadata: AccountInfo<'info>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub item_nft_program: Program<'info, ItemNft>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<CraftItem>, item_type: u8) -> Result<()> {
    require!(item_type < ITEM_COUNT as u8, CraftingError::InvalidItemType);

    let recipe = RECIPES[item_type as usize];

    // Перевіряємо, що передані мінти відповідають рецепту
    let (resource_id_0, _) = recipe[0];
    let (resource_id_1, _) = recipe[1];
    let (resource_id_2, _) = recipe[2];

    require!(
        ctx.accounts.resource_mint_0.key()
            == ctx.accounts.game_config.resource_mints[resource_id_0 as usize],
        CraftingError::InvalidResourceMint
    );
    require!(
        ctx.accounts.resource_mint_1.key()
            == ctx.accounts.game_config.resource_mints[resource_id_1 as usize],
        CraftingError::InvalidResourceMint
    );
    require!(
        ctx.accounts.resource_mint_2.key()
            == ctx.accounts.game_config.resource_mints[resource_id_2 as usize],
        CraftingError::InvalidResourceMint
    );

    // Перевіряємо баланси ресурсів
    require!(
        ctx.accounts.player_resource_account_0.amount >= recipe[0].1,
        CraftingError::InsufficientResources
    );
    require!(
        ctx.accounts.player_resource_account_1.amount >= recipe[1].1,
        CraftingError::InsufficientResources
    );
    require!(
        ctx.accounts.player_resource_account_2.amount >= recipe[2].1,
        CraftingError::InsufficientResources
    );

    let crafting_bump = ctx.accounts.crafting_authority.bump;
    let crafting_seeds: &[&[u8]] = &[b"crafting_authority", &[crafting_bump]];

    // Спалюємо перший інгредієнт
    resource_manager::cpi::burn_resource(
        CpiContext::new_with_signer(
            ctx.accounts.resource_manager_program.key(),
            BurnResource {
                game_config: ctx.accounts.game_config.to_account_info(),
                caller_authority: ctx.accounts.crafting_authority.to_account_info(),
                player: ctx.accounts.player.to_account_info(),
                mint: ctx.accounts.resource_mint_0.to_account_info(),
                player_token_account: ctx.accounts.player_resource_account_0.to_account_info(),
                token_2022_program: ctx.accounts.token_2022_program.to_account_info(),
            },
            &[crafting_seeds],
        ),
        resource_id_0,
        recipe[0].1,
    )?;

    // Спалюємо другий інгредієнт
    resource_manager::cpi::burn_resource(
        CpiContext::new_with_signer(
            ctx.accounts.resource_manager_program.key(),
            BurnResource {
                game_config: ctx.accounts.game_config.to_account_info(),
                caller_authority: ctx.accounts.crafting_authority.to_account_info(),
                player: ctx.accounts.player.to_account_info(),
                mint: ctx.accounts.resource_mint_1.to_account_info(),
                player_token_account: ctx.accounts.player_resource_account_1.to_account_info(),
                token_2022_program: ctx.accounts.token_2022_program.to_account_info(),
            },
            &[crafting_seeds],
        ),
        resource_id_1,
        recipe[1].1,
    )?;

    // Спалюємо третій інгредієнт
    resource_manager::cpi::burn_resource(
        CpiContext::new_with_signer(
            ctx.accounts.resource_manager_program.key(),
            BurnResource {
                game_config: ctx.accounts.game_config.to_account_info(),
                caller_authority: ctx.accounts.crafting_authority.to_account_info(),
                player: ctx.accounts.player.to_account_info(),
                mint: ctx.accounts.resource_mint_2.to_account_info(),
                player_token_account: ctx.accounts.player_resource_account_2.to_account_info(),
                token_2022_program: ctx.accounts.token_2022_program.to_account_info(),
            },
            &[crafting_seeds],
        ),
        resource_id_2,
        recipe[2].1,
    )?;

    // Назви та символи предметів для метаданих NFT
    let item_names = ["Шабля козака", "Посох старійшини", "Броня характерника", "Бойовий браслет"];
    let item_symbols = ["SABER", "STAFF", "ARMOR", "BRACELET"];
    let item_name = item_names[item_type as usize].to_string();
    let item_symbol = item_symbols[item_type as usize].to_string();
    let item_uri = format!("https://kozatsky-biznes.game/items/{}", item_type);

    // Мінтимо NFT через item_nft CPI
    item_nft::cpi::mint_nft_item(
        CpiContext::new_with_signer(
            ctx.accounts.item_nft_program.key(),
            MintNftItem {
                item_nft_config: ctx.accounts.item_nft_config.to_account_info(),
                nft_mint_authority: ctx.accounts.nft_mint_authority.to_account_info(),
                caller_authority: ctx.accounts.crafting_authority.to_account_info(),
                payer: ctx.accounts.player.to_account_info(),
                player: ctx.accounts.player.to_account_info(),
                nft_mint: ctx.accounts.nft_mint.to_account_info(),
                item_metadata: ctx.accounts.item_metadata.to_account_info(),
                player_nft_token_account: ctx.accounts.player_nft_token_account.to_account_info(),
                token_2022_program: ctx.accounts.token_2022_program.to_account_info(),
                associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &[crafting_seeds],
        ),
        item_type,
        item_name.clone(),
        item_symbol,
        item_uri,
    )?;

    msg!(
        "Гравець {} скрафтив предмет типу {} ({})",
        ctx.accounts.player.key(),
        item_type,
        item_name
    );
    Ok(())
}
