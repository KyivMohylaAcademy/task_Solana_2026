//! Crafting logic that consumes resource tokens and mints item NFTs.

use anchor_lang::prelude::*;
use anchor_spl::associated_token::{get_associated_token_address_with_program_id, AssociatedToken};
use anchor_spl::token::Token;
use anchor_spl::token_2022::{
    spl_token_2022::{extension::StateWithExtensions, state::Account as SplToken2022Account},
    Token2022,
};
use anchor_spl::token_interface::Mint;
use item_nft::program::ItemNft;
use resource_manager::{program::ResourceManager, GameConfig};
use shared::{
    recipe_for, GameErrorCode, ItemType, Recipe, ResourceType, GAME_CONFIG_SEED,
    ITEM_NFT_PROGRAM_ID, PROGRAM_AUTHORITY_SEED, RESOURCE_MANAGER_PROGRAM_ID,
    TOKEN_METADATA_PROGRAM_ID,
};

declare_id!("A14WMVRTuuS4JtVcg22BuiWHvhJx1ZhxJS5CrWfy2tHh");

/// Burns recipe resources and mints the corresponding item NFT.
#[program]
pub mod crafting {
    use super::*;

    /// Verifies shared bootstrap invariants for the crafting program.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        shared::validate_bootstrap_config()?;
        msg!("crafting bootstrap ready");
        Ok(())
    }

    /// Burns the exact recipe resources from the player and mints the crafted NFT.
    pub fn craft_item<'info>(
        ctx: Context<'_, '_, '_, 'info, CraftItem<'info>>,
        item_type: u8,
    ) -> Result<()> {
        shared::validate_bootstrap_config()?;

        let item_type = ItemType::from_index(usize::from(item_type))?;
        let recipe = recipe_for(item_type);
        let resource_accounts = ctx
            .accounts
            .collect_recipe_resource_accounts(recipe, ctx.remaining_accounts)?;
        ctx.accounts
            .validate_recipe_balances(recipe, &resource_accounts)?;

        let authority_bump = ctx.bumps.crafting_authority;
        let signer_seeds: &[&[u8]] = &[PROGRAM_AUTHORITY_SEED, &[authority_bump]];

        for resource_account in resource_accounts.iter() {
            resource_manager::cpi::burn_resource_from_player(
                CpiContext::new_with_signer(
                    ctx.accounts.resource_manager_program.to_account_info(),
                    resource_manager::cpi::accounts::BurnResourceFromPlayer {
                        player: ctx.accounts.owner.to_account_info(),
                        game_config: ctx.accounts.game_config.to_account_info(),
                        caller_authority: ctx.accounts.crafting_authority.to_account_info(),
                        resource_mint: resource_account.mint.clone(),
                        player_resource_token_account: resource_account.token_account.clone(),
                        token_program: ctx.accounts.resource_token_program.to_account_info(),
                    },
                    &[signer_seeds],
                ),
                resource_account.resource_type as u8,
                resource_account.required_amount,
            )?;
        }

        let descriptor = item_type.descriptor();
        item_nft::cpi::mint_item_nft(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                item_nft::cpi::accounts::MintItemNft {
                    owner: ctx.accounts.owner.to_account_info(),
                    game_config: ctx.accounts.game_config.to_account_info(),
                    caller_authority: ctx.accounts.crafting_authority.to_account_info(),
                    program_authority: ctx.accounts.item_nft_authority.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    item_metadata: ctx.accounts.item_metadata.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    master_edition: ctx.accounts.master_edition.to_account_info(),
                    owner_item_token_account: ctx
                        .accounts
                        .owner_item_token_account
                        .to_account_info(),
                    token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
                    token_program: ctx.accounts.item_token_program.to_account_info(),
                    associated_token_program: ctx
                        .accounts
                        .associated_token_program
                        .to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    sysvar_instructions: ctx.accounts.sysvar_instructions.to_account_info(),
                },
                &[signer_seeds],
            ),
            item_type as u8,
            descriptor.uri.to_string(),
            descriptor.name.to_string(),
            descriptor.symbol.to_string(),
        )
    }

    /// Test-oriented helper that proxies a resource burn through the crafting PDA.
    pub fn proxy_burn_resource(
        ctx: Context<ProxyBurnResource>,
        resource_type: u8,
        amount: u64,
    ) -> Result<()> {
        let authority_bump = ctx.bumps.crafting_authority;
        let signer_seeds: &[&[u8]] = &[PROGRAM_AUTHORITY_SEED, &[authority_bump]];

        resource_manager::cpi::burn_resource_from_player(
            CpiContext::new_with_signer(
                ctx.accounts.resource_manager_program.to_account_info(),
                resource_manager::cpi::accounts::BurnResourceFromPlayer {
                    player: ctx.accounts.owner.to_account_info(),
                    game_config: ctx.accounts.game_config.to_account_info(),
                    caller_authority: ctx.accounts.crafting_authority.to_account_info(),
                    resource_mint: ctx.accounts.resource_mint.to_account_info(),
                    player_resource_token_account: ctx
                        .accounts
                        .player_resource_token_account
                        .to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                &[signer_seeds],
            ),
            resource_type,
            amount,
        )
    }

    /// Test-oriented helper that proxies an NFT mint through the crafting PDA.
    pub fn proxy_mint_item_nft(
        ctx: Context<ProxyMintItemNft>,
        item_type: u8,
        uri: String,
        name: String,
        symbol: String,
    ) -> Result<()> {
        let authority_bump = ctx.bumps.crafting_authority;
        let signer_seeds: &[&[u8]] = &[PROGRAM_AUTHORITY_SEED, &[authority_bump]];

        item_nft::cpi::mint_item_nft(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                item_nft::cpi::accounts::MintItemNft {
                    owner: ctx.accounts.owner.to_account_info(),
                    game_config: ctx.accounts.game_config.to_account_info(),
                    caller_authority: ctx.accounts.crafting_authority.to_account_info(),
                    program_authority: ctx.accounts.item_nft_authority.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    item_metadata: ctx.accounts.item_metadata.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    master_edition: ctx.accounts.master_edition.to_account_info(),
                    owner_item_token_account: ctx
                        .accounts
                        .owner_item_token_account
                        .to_account_info(),
                    token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    associated_token_program: ctx
                        .accounts
                        .associated_token_program
                        .to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    sysvar_instructions: ctx.accounts.sysvar_instructions.to_account_info(),
                },
                &[signer_seeds],
            ),
            item_type,
            uri,
            name,
            symbol,
        )
    }
}

/// Empty bootstrap context used to confirm the program is deployed and configured.
#[derive(Accounts)]
pub struct Initialize {}

/// Accounts required to craft one item NFT from resource token accounts.
#[derive(Accounts)]
pub struct CraftItem<'info> {
    /// Player wallet paying rent and authorizing resource burns.
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump = game_config.bump
    )]
    /// Shared config account containing canonical mint addresses.
    pub game_config: Box<Account<'info, GameConfig>>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: this PDA signs CPIs into resource_manager and item_nft.
    /// Crafting PDA that signs CPIs into `resource_manager` and `item_nft`.
    pub crafting_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        seeds::program = ITEM_NFT_PROGRAM_ID,
        bump
    )]
    /// CHECK: item_nft validates this PDA again before Metaplex CPI.
    /// Item-nft PDA used as mint/update authority during Metaplex CPIs.
    pub item_nft_authority: UncheckedAccount<'info>,
    /// Newly generated NFT mint signer.
    #[account(mut)]
    pub mint: Signer<'info>,
    #[account(mut)]
    /// CHECK: item_nft initializes and validates this PDA.
    /// PDA that will store gameplay metadata for the minted NFT.
    pub item_metadata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: item_nft validates the Metaplex metadata PDA.
    /// Metaplex metadata PDA for the minted NFT.
    pub metadata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: item_nft validates the Metaplex master edition PDA.
    /// Metaplex master edition PDA for the minted NFT.
    pub master_edition: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: item_nft validates the expected owner ATA.
    /// Owner ATA that will receive the crafted NFT.
    pub owner_item_token_account: UncheckedAccount<'info>,
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    /// CHECK: constrained to the canonical Metaplex Token Metadata program.
    /// Canonical Metaplex Token Metadata program.
    pub token_metadata_program: UncheckedAccount<'info>,
    /// CPI interface for resource burns.
    pub resource_manager_program: Program<'info, ResourceManager>,
    /// CPI interface for item NFT minting.
    pub item_nft_program: Program<'info, ItemNft>,
    /// Token-2022 program that owns resource mints and ATAs.
    pub resource_token_program: Program<'info, Token2022>,
    /// SPL Token program that owns item NFT mints and ATAs.
    pub item_token_program: Program<'info, Token>,
    /// Associated token program used during NFT minting.
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// System program used during NFT minting.
    pub system_program: Program<'info, System>,
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: constrained to the instructions sysvar address.
    /// Instructions sysvar required by Metaplex CPIs.
    pub sysvar_instructions: UncheckedAccount<'info>,
}

impl<'info> CraftItem<'info> {
    /// Parses and validates remaining accounts into ordered recipe resource pairs.
    fn collect_recipe_resource_accounts(
        &self,
        recipe: &Recipe,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<Vec<RecipeResourceAccount<'info>>> {
        let expected_resource_types = ResourceType::ALL
            .into_iter()
            .filter(|resource_type| recipe.requires(*resource_type) > 0)
            .collect::<Vec<_>>();

        require!(
            remaining_accounts.len() == expected_resource_types.len() * 2,
            GameErrorCode::InvalidCraftingResourceAccounts
        );

        let mut recipe_accounts = Vec::with_capacity(expected_resource_types.len());
        for (pair_index, resource_type) in expected_resource_types.into_iter().enumerate() {
            let mint = remaining_accounts[pair_index * 2].clone();
            let token_account = remaining_accounts[(pair_index * 2) + 1].clone();

            self.validate_resource_mint(resource_type, &mint)?;
            self.validate_resource_token_account(&mint, &token_account)?;

            recipe_accounts.push(RecipeResourceAccount {
                resource_type,
                required_amount: recipe.requires(resource_type),
                mint,
                token_account,
            });
        }

        Ok(recipe_accounts)
    }

    /// Checks that each provided resource account contains enough balance for the recipe.
    fn validate_recipe_balances(
        &self,
        _recipe: &Recipe,
        resource_accounts: &[RecipeResourceAccount<'info>],
    ) -> Result<()> {
        for resource_account in resource_accounts {
            let balance = self.resource_balance(resource_account)?;
            require!(
                balance >= resource_account.required_amount,
                GameErrorCode::InsufficientResourcesForRecipe
            );
        }

        Ok(())
    }

    /// Verifies that a provided mint matches the expected resource PDA.
    fn validate_resource_mint(
        &self,
        resource_type: ResourceType,
        mint: &AccountInfo<'info>,
    ) -> Result<()> {
        let expected_mint = self.game_config.resource_mints[resource_type.as_index()];
        require_keys_eq!(
            mint.key(),
            expected_mint,
            GameErrorCode::ResourceMintAddressMismatch
        );

        Ok(())
    }

    /// Verifies that a provided ATA belongs to the crafter for the given resource mint.
    fn validate_resource_token_account(
        &self,
        mint: &AccountInfo<'info>,
        token_account: &AccountInfo<'info>,
    ) -> Result<()> {
        let expected_token_account = get_associated_token_address_with_program_id(
            &self.owner.key(),
            &mint.key(),
            &self.resource_token_program.key(),
        );
        require_keys_eq!(
            token_account.key(),
            expected_token_account,
            GameErrorCode::InvalidResourceTokenAccount
        );

        Ok(())
    }

    /// Reads the Token-2022 balance from a raw remaining account.
    fn resource_balance(&self, resource_account: &RecipeResourceAccount<'info>) -> Result<u64> {
        let token_account = &resource_account.token_account;
        if token_account.data_is_empty() {
            return Ok(0);
        }

        require_keys_eq!(
            *token_account.owner,
            self.resource_token_program.key(),
            GameErrorCode::InvalidResourceTokenAccount
        );

        let account_data = token_account.try_borrow_data()?;
        let parsed_account = StateWithExtensions::<SplToken2022Account>::unpack(&account_data)
            .map_err(|_| error!(GameErrorCode::InvalidResourceTokenAccount))?;

        require_keys_eq!(
            Pubkey::from(parsed_account.base.mint),
            resource_account.mint.key(),
            GameErrorCode::InvalidResourceTokenAccount
        );
        require_keys_eq!(
            Pubkey::from(parsed_account.base.owner),
            self.owner.key(),
            GameErrorCode::InvalidResourceTokenAccount
        );

        Ok(parsed_account.base.amount)
    }
}

/// Internal representation of one recipe resource pair from `remaining_accounts`.
struct RecipeResourceAccount<'info> {
    resource_type: ResourceType,
    required_amount: u64,
    mint: AccountInfo<'info>,
    token_account: AccountInfo<'info>,
}

/// Accounts required by tests to burn a player resource through the crafting PDA.
#[derive(Accounts)]
pub struct ProxyBurnResource<'info> {
    /// Player wallet authorizing the burn.
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump = game_config.bump
    )]
    /// Shared config used by `resource_manager` during burn validation.
    pub game_config: Account<'info, GameConfig>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: this PDA signs the CPI into resource_manager.
    /// Crafting PDA that signs the burn CPI.
    pub crafting_authority: UncheckedAccount<'info>,
    /// Resource mint whose tokens will be burned.
    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    /// CHECK: resource_manager validates the Token-2022 account ownership and mint.
    /// Player ATA supplying the burned tokens.
    pub player_resource_token_account: UncheckedAccount<'info>,
    /// CPI interface for the resource manager program.
    pub resource_manager_program: Program<'info, ResourceManager>,
    /// Token-2022 program used for the burn CPI.
    pub token_program: Program<'info, Token2022>,
}

/// Accounts required by tests to mint an item NFT through the crafting PDA.
#[derive(Accounts)]
pub struct ProxyMintItemNft<'info> {
    /// Player wallet paying rent and receiving the NFT.
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump = game_config.bump
    )]
    /// Shared config used by `item_nft` for item-type validation.
    pub game_config: Account<'info, GameConfig>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: this PDA signs the CPI into item_nft.
    /// Crafting PDA that signs the item-nft CPI.
    pub crafting_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        seeds::program = ITEM_NFT_PROGRAM_ID,
        bump
    )]
    /// CHECK: item_nft validates this PDA again before Metaplex CPI.
    /// Item-nft PDA used as mint/update authority during Metaplex CPIs.
    pub item_nft_authority: UncheckedAccount<'info>,
    /// Newly generated NFT mint signer.
    #[account(mut)]
    pub mint: Signer<'info>,
    #[account(mut)]
    /// CHECK: item_nft initializes and validates this PDA.
    /// PDA that will store gameplay metadata for the minted NFT.
    pub item_metadata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: item_nft validates the Metaplex metadata PDA.
    /// Metaplex metadata PDA for the minted NFT.
    pub metadata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: item_nft validates the Metaplex master edition PDA.
    /// Metaplex master edition PDA for the minted NFT.
    pub master_edition: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: item_nft validates the expected owner ATA.
    /// Owner ATA that will receive the NFT.
    pub owner_item_token_account: UncheckedAccount<'info>,
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    /// CHECK: constrained to the canonical Metaplex Token Metadata program.
    /// Canonical Metaplex Token Metadata program.
    pub token_metadata_program: UncheckedAccount<'info>,
    /// CPI interface for the item-nft program.
    pub item_nft_program: Program<'info, ItemNft>,
    /// SPL Token program that owns the NFT mint and ATA.
    pub token_program: Program<'info, Token>,
    /// Associated token program used during NFT minting.
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// System program used during NFT minting.
    pub system_program: Program<'info, System>,
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: constrained to the instructions sysvar address.
    /// Instructions sysvar required by Metaplex CPIs.
    pub sysvar_instructions: UncheckedAccount<'info>,
}
