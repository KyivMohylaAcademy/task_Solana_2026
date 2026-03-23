use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, program::invoke_signed, system_instruction};
use spl_token_2022::{extension::ExtensionType, instruction::initialize_mint2};
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("8aVvoB3XXALKiaNnGbq6KC76aHA82jiAMRk7tnoPx22U");

#[program]
pub mod resource_manager {
    use super::*;

    /// Initialize the game configuration with admin-tunable parameters.
    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        item_prices: [u64; 4],
        rarity_weights: [u8; 6],
        search_cooldown: i64,
        search_program: Pubkey,
        crafting_program: Pubkey,
        marketplace_program: Pubkey,
    ) -> Result<()> {
        let total: u16 = rarity_weights.iter().map(|&w| w as u16).sum();
        require!(total == 100, GameError::InvalidRarityWeights);
        require!(search_cooldown > 0, GameError::InvalidCooldown);

        let config = &mut ctx.accounts.game_config;
        config.admin = ctx.accounts.admin.key();
        config.item_prices = item_prices;
        config.rarity_weights = rarity_weights;
        config.search_cooldown = search_cooldown;
        config.search_program = search_program;
        config.crafting_program = crafting_program;
        config.marketplace_program = marketplace_program;
        config.resource_count = 0;
        config.bump = ctx.bumps.game_config;
        config.mint_authority_bump = ctx.bumps.mint_authority;
        Ok(())
    }

    /// Create a new resource mint (Token-2022 + MetadataPointer).
    /// Must be called in order: id 0, 1, 2, 3, 4, 5.
    pub fn initialize_resource(
        ctx: Context<InitializeResource>,
        id: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        require!(id < RESOURCE_COUNT as u8, GameError::InvalidResourceId);
        require!(
            id == ctx.accounts.game_config.resource_count,
            GameError::OutOfOrder
        );

        let mint_key = ctx.accounts.mint.key();
        let authority_key = ctx.accounts.mint_authority.key();

        let mint_space =
            ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
                ExtensionType::MetadataPointer,
            ])
            .map_err(|_| GameError::SpaceCalculationFailed)?;

        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(mint_space);

        invoke(
            &system_instruction::create_account(
                &ctx.accounts.admin.key(),
                &mint_key,
                lamports,
                mint_space as u64,
                &spl_token_2022::ID,
            ),
            &[
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.mint.to_account_info(),
            ],
        )?;

        invoke(
            &spl_token_2022::extension::metadata_pointer::instruction::initialize(
                &spl_token_2022::ID,
                &mint_key,
                Some(authority_key),
                Some(mint_key),
            )?,
            &[ctx.accounts.mint.to_account_info()],
        )?;

        invoke(
            &initialize_mint2(&spl_token_2022::ID, &mint_key, &authority_key, None, 0)?,
            &[ctx.accounts.mint.to_account_info()],
        )?;

        let token_metadata = TokenMetadata {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            mint: mint_key,
            update_authority: Some(authority_key).try_into().unwrap(),
            additional_metadata: vec![],
        };
        let metadata_data_len = token_metadata.get_packed_len().unwrap_or(256);
        let new_total_space = mint_space + 12 + metadata_data_len;
        let extra_lamports = rent
            .minimum_balance(new_total_space)
            .saturating_sub(lamports);

        if extra_lamports > 0 {
            invoke(
                &system_instruction::transfer(
                    &ctx.accounts.admin.key(),
                    &mint_key,
                    extra_lamports,
                ),
                &[
                    ctx.accounts.admin.to_account_info(),
                    ctx.accounts.mint.to_account_info(),
                ],
            )?;
        }

        let auth_bump = ctx.accounts.game_config.mint_authority_bump;
        let authority_seeds: &[&[u8]] = &[b"mint_authority", &[auth_bump]];
        invoke_signed(
            &spl_token_metadata_interface::instruction::initialize(
                &spl_token_2022::ID,
                &mint_key,
                &authority_key,
                &mint_key,
                &authority_key,
                name,
                symbol,
                uri,
            ),
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
            ],
            &[authority_seeds],
        )?;

        let config = &mut ctx.accounts.game_config;
        config.resource_mints[id as usize] = mint_key;
        config.resource_count += 1;
        Ok(())
    }

    /// Mint resources to a player. CPI-gated: only callable by the search program.
    pub fn mint_resource(ctx: Context<MintResource>, resource_id: u8, amount: u64) -> Result<()> {
        require!(
            resource_id < RESOURCE_COUNT as u8,
            GameError::InvalidResourceId
        );
        require!(amount > 0, GameError::InvalidAmount);

        let bump = ctx.accounts.game_config.mint_authority_bump;
        let seeds: &[&[u8]] = &[b"mint_authority", &[bump]];

        anchor_spl::token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_2022::MintTo {
                    mint: ctx.accounts.resource_mint.to_account_info(),
                    to: ctx.accounts.player_ata.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )
    }

    /// Burn resources from a player. CPI-gated: only callable by the crafting program.
    pub fn burn_resource(ctx: Context<BurnResource>, resource_id: u8, amount: u64) -> Result<()> {
        require!(
            resource_id < RESOURCE_COUNT as u8,
            GameError::InvalidResourceId
        );
        require!(amount > 0, GameError::InvalidAmount);

        anchor_spl::token_2022::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_2022::Burn {
                    mint: ctx.accounts.resource_mint.to_account_info(),
                    from: ctx.accounts.player_ata.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            amount,
        )
    }

    /// Admin: update rarity weights for resource drops.
    pub fn update_rarity_weights(ctx: Context<AdminOnly>, new_weights: [u8; 6]) -> Result<()> {
        let total: u16 = new_weights.iter().map(|&w| w as u16).sum();
        require!(total == 100, GameError::InvalidRarityWeights);
        ctx.accounts.game_config.rarity_weights = new_weights;
        Ok(())
    }

    /// Admin: update search cooldown in seconds.
    pub fn update_search_cooldown(ctx: Context<AdminOnly>, new_cooldown: i64) -> Result<()> {
        require!(new_cooldown > 0, GameError::InvalidCooldown);
        ctx.accounts.game_config.search_cooldown = new_cooldown;
        Ok(())
    }
}
