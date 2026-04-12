use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use item_nft::cpi::accounts::CreateItemNft;
use item_nft::cpi::create_item_nft;
use item_nft::program::ItemNft;
use item_nft::state::{ITEM_NAMES, ITEM_SYMBOLS};
use resource_manager::cpi::accounts::BurnResource;
use resource_manager::cpi::burn_resource;
use resource_manager::program::ResourceManager;
use resource_manager::state::GameConfig;

use crate::constants::*;
use crate::errors::CraftingError;

pub const GAME_CONFIG_SEED: &[u8] = b"game_config";

/// Акаунти для крафту предмета.
/// remaining_accounts: пари [mint, token_account] для кожного інгредієнту.
/// Порядок: [mint_0, ta_0, mint_1, ta_1, mint_2, ta_2] — максимум 3 інгредієнти.
#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CraftItem<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    /// GameConfig з resource_manager.
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager_program.key(),
    )]
    pub game_config: Account<'info, GameConfig>,

    // NFT акаунти
    /// ItemMetadata PDA — буде створена через item_nft CPI.
    /// CHECK: Ініціалізується через item_nft::create_item_nft CPI.
    #[account(mut)]
    pub item_metadata: UncheckedAccount<'info>,

    /// NFT мінт — новий keypair, підписує для item_nft CPI.
    #[account(mut)]
    pub nft_mint: Signer<'info>,

    /// Токен-акаунт гравця для NFT.
    /// CHECK: Ініціалізується через item_nft CPI.
    #[account(mut)]
    pub player_nft_account: UncheckedAccount<'info>,

    /// CHECK: Metaplex Metadata акаунт.
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK: Metaplex Master Edition PDA.
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: Metaplex Token Metadata program.
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,

    /// CHECK: Sysvar instructions — Sysvar1nstructions1111111111111111111111111.
    pub sysvar_instructions: UncheckedAccount<'info>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub item_nft_program: Program<'info, ItemNft>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'info, CraftItem<'info>>, item_type: u8, metadata_uri: String) -> Result<()> {
    require!(item_type < ITEM_TYPE_COUNT, CraftingError::InvalidItemType);

    let recipe = RECIPES[item_type as usize];

    // Витягуємо всі AccountInfo наперед щоб уникнути конфліктів лайфтаймів
    let game_config_info = ctx.accounts.game_config.to_account_info();
    let resource_mints = ctx.accounts.game_config.resource_mints;
    let player_info = ctx.accounts.player.to_account_info();
    let token_program_info = ctx.accounts.token_program.to_account_info();
    let item_metadata_info = ctx.accounts.item_metadata.to_account_info();
    let nft_mint_info = ctx.accounts.nft_mint.to_account_info();
    let player_nft_info = ctx.accounts.player_nft_account.to_account_info();
    let metadata_info = ctx.accounts.metadata_account.to_account_info();
    let master_edition_info = ctx.accounts.master_edition.to_account_info();
    let sysvar_instructions_info = ctx.accounts.sysvar_instructions.to_account_info();
    let token_meta_prog_info = ctx.accounts.token_metadata_program.to_account_info();
    let assoc_token_info = ctx.accounts.associated_token_program.to_account_info();
    let system_info = ctx.accounts.system_program.to_account_info();

    // remaining_accounts: [mint_0, ta_0, mint_1, ta_1, mint_2, ta_2]
    let remaining: Vec<AccountInfo> = ctx.remaining_accounts.iter().cloned().collect();

    // Спалюємо ресурси за рецептом
    let mut ra_index: usize = 0;
    for (resource_id, amount) in recipe.iter() {
        if *amount == 0 {
            continue;
        }
        require!(ra_index + 1 < remaining.len(), CraftingError::InsufficientResources);

        let mint_info = remaining[ra_index].clone();
        let token_info = remaining[ra_index + 1].clone();
        ra_index += 2;

        require!(
            resource_mints[*resource_id as usize] == mint_info.key(),
            CraftingError::InsufficientResources
        );

        burn_resource(
            CpiContext::new(
                resource_manager::ID,
                BurnResource {
                    game_config: game_config_info.clone(),
                    resource_mint: mint_info,
                    player_token_account: token_info,
                    player: player_info.clone(),
                    token_program: token_program_info.clone(),
                },
            ),
            *resource_id,
            *amount,
        )?;
    }

    // Мінтимо NFT через item_nft CPI
    create_item_nft(
        CpiContext::new(
            item_nft::ID,
            CreateItemNft {
                player: player_info,
                item_metadata: item_metadata_info,
                nft_mint: nft_mint_info,
                player_nft_account: player_nft_info,
                metadata_account: metadata_info,
                master_edition: master_edition_info,
                token_metadata_program: token_meta_prog_info,
                sysvar_instructions: sysvar_instructions_info,
                token_program: token_program_info,
                associated_token_program: assoc_token_info,
                system_program: system_info,
            },
        ),
        item_type,
        ITEM_NAMES[item_type as usize].to_string(),
        ITEM_SYMBOLS[item_type as usize].to_string(),
        metadata_uri,
    )?;

    msg!("Предмет типу {} скрафтено!", item_type);
    Ok(())
}
