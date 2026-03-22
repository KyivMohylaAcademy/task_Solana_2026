use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::Metadata;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use item_nft::cpi::accounts::CreateItem;
use item_nft::program::ItemNft;
use resource_manager::cpi::accounts::BurnResource;
use resource_manager::program::ResourceManager;
use resource_manager::{self as resource_mgr, GameConfig};

declare_id!("DmciAbQkk3a26UE8USvCi1VwZRs4DjXxNszqfo3nNYvv");

pub const ITEM_SHABLIA: u8 = 0;
pub const ITEM_POSOKH: u8 = 1;
pub const ITEM_BRONYA: u8 = 2;
pub const ITEM_BRASLET: u8 = 3;

// [wood, iron, gold, leather, stone, diamond]
pub const RECIPES: [[u64; 6]; 4] = [
    [1, 3, 0, 1, 0, 0], // Шабля козака
    [2, 0, 1, 0, 0, 1], // Посох старійшини
    [0, 2, 1, 4, 0, 0], // Броня характерника
    [0, 4, 2, 0, 0, 2], // Бойовий браслет
];

#[program]
pub mod crafting {
    use super::*;

    pub fn craft_item(ctx: Context<CraftItem>, item_type: u8) -> Result<()> {
        require!(
            (item_type as usize) < RECIPES.len(),
            CraftingError::InvalidItemType
        );

        let recipe = RECIPES[item_type as usize];

        let resource_mints = [
            ctx.accounts.wood_mint.to_account_info(),
            ctx.accounts.iron_mint.to_account_info(),
            ctx.accounts.gold_mint.to_account_info(),
            ctx.accounts.leather_mint.to_account_info(),
            ctx.accounts.stone_mint.to_account_info(),
            ctx.accounts.diamond_mint.to_account_info(),
        ];
        let resource_token_accounts = [
            ctx.accounts.player_wood_account.to_account_info(),
            ctx.accounts.player_iron_account.to_account_info(),
            ctx.accounts.player_gold_account.to_account_info(),
            ctx.accounts.player_leather_account.to_account_info(),
            ctx.accounts.player_stone_account.to_account_info(),
            ctx.accounts.player_diamond_account.to_account_info(),
        ];

        let player_info = ctx.accounts.player.to_account_info();
        let token_2022_info = ctx.accounts.token_2022_program.to_account_info();
        let rm_info = ctx.accounts.resource_manager_program.to_account_info();

        for i in 0..6 {
            if recipe[i] > 0 {
                let cpi_accounts = BurnResource {
                    resource_mint: resource_mints[i].clone(),
                    player_token_account: resource_token_accounts[i].clone(),
                    player: player_info.clone(),
                    token_program: token_2022_info.clone(),
                };

                let cpi_ctx = CpiContext::new(rm_info.clone(), cpi_accounts);
                resource_mgr::cpi::burn_resource(cpi_ctx, i as u8, recipe[i])?;
            }
        }

        let cpi_accounts = CreateItem {
            item_metadata: ctx.accounts.item_metadata.to_account_info(),
            item_mint: ctx.accounts.item_mint.to_account_info(),
            player_item_account: ctx.accounts.player_item_account.to_account_info(),
            metadata_account: ctx.accounts.metadata_account.to_account_info(),
            player: player_info.clone(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
            metadata_program: ctx.accounts.metadata_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.item_nft_program.to_account_info(),
            cpi_accounts,
        );

        item_nft::cpi::create_item(cpi_ctx, item_type)?;

        msg!("Crafting: Created item type {}", item_type);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CraftItem<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    pub game_config: Box<Account<'info, GameConfig>>,

    #[account(mut, address = game_config.resource_mints[0])]
    pub wood_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[1])]
    pub iron_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[2])]
    pub gold_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[3])]
    pub leather_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[4])]
    pub stone_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[5])]
    pub diamond_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut, token::mint = wood_mint, token::authority = player)]
    pub player_wood_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, token::mint = iron_mint, token::authority = player)]
    pub player_iron_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, token::mint = gold_mint, token::authority = player)]
    pub player_gold_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, token::mint = leather_mint, token::authority = player)]
    pub player_leather_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, token::mint = stone_mint, token::authority = player)]
    pub player_stone_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut, token::mint = diamond_mint, token::authority = player)]
    pub player_diamond_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: created by item_nft CPI
    #[account(mut)]
    pub item_metadata: AccountInfo<'info>,

    #[account(mut)]
    pub item_mint: Signer<'info>,

    /// CHECK: created by item_nft CPI
    #[account(mut)]
    pub player_item_account: AccountInfo<'info>,

    /// CHECK: Metaplex metadata
    #[account(mut)]
    pub metadata_account: AccountInfo<'info>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub item_nft_program: Program<'info, ItemNft>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_2022_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[error_code]
pub enum CraftingError {
    #[msg("Invalid item type. Must be 0-3.")]
    InvalidItemType,
    #[msg("Insufficient resources for crafting.")]
    InsufficientResources,
}
