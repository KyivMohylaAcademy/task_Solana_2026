use anchor_lang::prelude::*;
use anchor_spl::metadata;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use game_common::{
    recipe_for, CRAFTING_AUTHORITY_SEED, GAME_CONFIG_SEED, ITEM_AUTHORITY_SEED, ITEM_METADATA_SEED,
    ITEM_MINT_SEED,
};

declare_id!("EZdAg3bGtT4FwK9xcpUKM6UuJzYB8BMvXyKoHz3mS986");

#[program]
pub mod crafting {
    use super::*;

    /// Burns the required resources and mints the crafted NFT.
    pub fn craft_item<'info>(
        ctx: Context<'_, '_, '_, 'info, CraftItem<'info>>,
        item_type: u8,
        mint_seed: [u8; 32],
        uri: String,
    ) -> Result<()> {
        let recipe = recipe_for(item_type).ok_or_else(|| error!(ErrorCode::InvalidItemType))?;
        let required_resources: Vec<(usize, u64)> = recipe
            .iter()
            .enumerate()
            .filter_map(|(index, amount)| (*amount > 0).then_some((index, *amount)))
            .collect();
        require!(
            ctx.remaining_accounts.len() == required_resources.len() * 2,
            ErrorCode::InvalidRemainingAccounts
        );
        let game_config_key = ctx.accounts.game_config.key();
        let crafting_signer: &[&[u8]] = &[
            CRAFTING_AUTHORITY_SEED,
            game_config_key.as_ref(),
            &[ctx.bumps.crafting_authority],
        ];

        for (pair_index, (resource_index, amount)) in required_resources.iter().enumerate() {
            let mint = ctx.remaining_accounts[pair_index * 2].clone();
            let token_account = ctx.remaining_accounts[pair_index * 2 + 1].clone();
            require_keys_eq!(
                ctx.accounts.game_config.resource_mints[*resource_index],
                mint.key(),
                ErrorCode::InvalidMintConfiguration
            );

            resource_manager::cpi::burn_resource(
                CpiContext::new_with_signer(
                    ctx.accounts.resource_manager_program.to_account_info(),
                    resource_manager::cpi::accounts::BurnResource {
                        authority: ctx.accounts.crafting_authority.to_account_info(),
                        owner: ctx.accounts.player.to_account_info(),
                        game_config: ctx.accounts.game_config.to_account_info(),
                        mint,
                        source: token_account,
                        mint_authority: ctx.accounts.resource_authority.to_account_info(),
                        token_program: ctx.accounts.token_2022_program.to_account_info(),
                    },
                    &[crafting_signer],
                ),
                *resource_index as u8,
                *amount,
            )?;
        }

        item_nft::cpi::mint_item(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                item_nft::cpi::accounts::MintItem {
                    player: ctx.accounts.player.to_account_info(),
                    game_config: ctx.accounts.game_config.to_account_info(),
                    crafting_authority: ctx.accounts.crafting_authority.to_account_info(),
                    item_authority: ctx.accounts.item_authority.to_account_info(),
                    item_mint: ctx.accounts.item_mint.to_account_info(),
                    player_item_account: ctx.accounts.player_item_account.to_account_info(),
                    item_metadata: ctx.accounts.item_metadata.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    master_edition: ctx.accounts.master_edition.to_account_info(),
                    metadata_program: ctx.accounts.metadata_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    associated_token_program: ctx
                        .accounts
                        .associated_token_program
                        .to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[crafting_signer],
            ),
            item_type,
            mint_seed,
            uri,
        )
    }
}

#[derive(Accounts)]
#[instruction(item_type: u8, mint_seed: [u8; 32], uri: String)]
pub struct CraftItem<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID
    )]
    pub game_config: Account<'info, resource_manager::GameConfig>,
    /// CHECK: PDA signer that authorizes CPI calls from crafting into item_nft.
    #[account(seeds = [CRAFTING_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub crafting_authority: UncheckedAccount<'info>,
    pub resource_manager_program: Program<'info, resource_manager::program::ResourceManager>,
    /// CHECK: PDA validated by seeds against resource_manager and used as freeze authority.
    #[account(seeds = [game_common::RESOURCE_AUTHORITY_SEED, game_config.key().as_ref()], bump, seeds::program = resource_manager::ID)]
    pub resource_authority: UncheckedAccount<'info>,
    pub item_nft_program: Program<'info, item_nft::program::ItemNft>,
    /// CHECK: PDA validated by seeds against the item_nft program.
    #[account(mut, seeds = [ITEM_AUTHORITY_SEED, game_config.key().as_ref()], bump, seeds::program = item_nft::ID)]
    pub item_authority: UncheckedAccount<'info>,
    /// CHECK: PDA validated by seeds; the item_nft program initializes it as a mint.
    #[account(mut, seeds = [ITEM_MINT_SEED, player.key().as_ref(), mint_seed.as_ref()], bump, seeds::program = item_nft::ID)]
    pub item_mint: UncheckedAccount<'info>,
    /// CHECK: The item_nft CPI creates or reuses the player's ATA for the crafted NFT.
    #[account(mut)]
    pub player_item_account: UncheckedAccount<'info>,
    /// CHECK: PDA validated by seeds; the item_nft CPI initializes it.
    #[account(mut, seeds = [ITEM_METADATA_SEED, item_mint.key().as_ref()], bump, seeds::program = item_nft::ID)]
    pub item_metadata: UncheckedAccount<'info>,
    /// CHECK: PDA address is validated inside item_nft against Metaplex derivation.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: PDA address is validated inside item_nft against Metaplex derivation.
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: Fixed Metaplex Token Metadata program id.
    #[account(address = metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unknown item type.")]
    InvalidItemType,
    #[msg("Resource mint list does not match the game config.")]
    InvalidMintConfiguration,
    #[msg("Remaining accounts do not match the recipe layout.")]
    InvalidRemainingAccounts,
}
