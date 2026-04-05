//! Canonical resource and game-configuration management for the crafting game.

use anchor_lang::prelude::*;
use anchor_spl::token_2022::{burn, mint_to, Burn, MintTo, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};
use shared::{
    find_resource_mint_address, GameErrorCode, ResourceType, CRAFTING_PROGRAM_ID, GAME_CONFIG_SEED,
    ITEM_COUNT, PROGRAM_AUTHORITY_SEED, RESOURCE_COUNT, RESOURCE_MINT_SEED, SEARCH_PROGRAM_ID,
};

declare_id!("CwwxNgkg1s8rjRAAN9zcvLgBCBhXTvCu4L1oAupBqiTe");

/// Initializes shared game state and manages resource mint authorities.
#[program]
pub mod resource_manager {
    use super::*;

    /// Creates the singleton `GameConfig` account with canonical mint PDAs and item prices.
    pub fn initialize_game_config(
        ctx: Context<InitializeGameConfig>,
        reward_token_mint: Pubkey,
        item_prices: [u64; ITEM_COUNT],
    ) -> Result<()> {
        shared::validate_bootstrap_config()?;

        let game_config = &mut ctx.accounts.game_config;
        let resource_mints = std::array::from_fn(|index| {
            let resource_type = ResourceType::from_index(index)
                .expect("resource index is bounded by RESOURCE_COUNT");
            find_resource_mint_address(&crate::ID, resource_type).0
        });

        game_config.admin = ctx.accounts.admin.key();
        game_config.resource_mints = resource_mints;
        game_config.reward_token_mint = reward_token_mint;
        game_config.item_prices = item_prices;
        game_config.bump = ctx.bumps.game_config;

        Ok(())
    }

    /// Creates a Token-2022 mint for one resource type at its canonical PDA.
    pub fn initialize_resource_mint(
        ctx: Context<InitializeResourceMint>,
        resource_type: u8,
        _name: String,
        _symbol: String,
        _uri: String,
    ) -> Result<()> {
        let resource_type = ResourceType::from_index(usize::from(resource_type))?;
        let expected_mint = ctx.accounts.game_config.resource_mints[resource_type.as_index()];

        require_keys_eq!(
            ctx.accounts.resource_mint.key(),
            expected_mint,
            GameErrorCode::ResourceMintAddressMismatch
        );

        Ok(())
    }

    /// Mints resource tokens to a player when invoked by the authorized search program.
    pub fn mint_resource_to_player(
        ctx: Context<MintResourceToPlayer>,
        resource_type: u8,
        amount: u64,
    ) -> Result<()> {
        let resource_type = ResourceType::from_index(usize::from(resource_type))?;
        require!(amount > 0, GameErrorCode::InvalidTokenAmount);

        let expected_mint = ctx.accounts.game_config.resource_mints[resource_type.as_index()];
        require_keys_eq!(
            ctx.accounts.resource_mint.key(),
            expected_mint,
            GameErrorCode::ResourceMintAddressMismatch
        );
        require!(
            ctx.accounts.caller_authority.is_signer,
            GameErrorCode::UnauthorizedResourceMintCaller
        );

        let authority_bump = ctx.bumps.program_authority;
        let signer_seeds: &[&[u8]] = &[PROGRAM_AUTHORITY_SEED, &[authority_bump]];
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.resource_mint.to_account_info(),
                    to: ctx.accounts.player_resource_token_account.to_account_info(),
                    authority: ctx.accounts.program_authority.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
        )?;

        Ok(())
    }

    /// Burns resource tokens from a player when invoked by the authorized crafting program.
    pub fn burn_resource_from_player(
        ctx: Context<BurnResourceFromPlayer>,
        resource_type: u8,
        amount: u64,
    ) -> Result<()> {
        let resource_type = ResourceType::from_index(usize::from(resource_type))?;
        require!(amount > 0, GameErrorCode::InvalidTokenAmount);

        let expected_mint = ctx.accounts.game_config.resource_mints[resource_type.as_index()];
        require_keys_eq!(
            ctx.accounts.resource_mint.key(),
            expected_mint,
            GameErrorCode::ResourceMintAddressMismatch
        );
        require!(
            ctx.accounts.caller_authority.is_signer,
            GameErrorCode::UnauthorizedResourceBurnCaller
        );

        burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.resource_mint.to_account_info(),
                    from: ctx.accounts.player_resource_token_account.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }
}

/// Accounts required to initialize the global `GameConfig` state.
#[derive(Accounts)]
pub struct InitializeGameConfig<'info> {
    /// Bootstrap admin paying rent and becoming the game owner.
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + GameConfig::INIT_SPACE,
        seeds = [GAME_CONFIG_SEED],
        bump
    )]
    /// Singleton configuration account for resource mint PDAs and item prices.
    pub game_config: Account<'info, GameConfig>,
    /// System program used to create the account.
    pub system_program: Program<'info, System>,
}

/// Accounts required to create one of the six canonical resource mints.
#[derive(Accounts)]
#[instruction(resource_type: u8)]
pub struct InitializeResourceMint<'info> {
    /// Bootstrap admin paying rent for the mint.
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        constraint = usize::from(resource_type) < RESOURCE_COUNT @ GameErrorCode::InvalidResourceTypeIndex,
        constraint = game_config.admin == admin.key() @ GameErrorCode::UnauthorizedAdmin
    )]
    /// Shared game config used to authorize the admin and canonical mint address.
    pub game_config: Account<'info, GameConfig>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: this PDA is only used as the deterministic mint/freeze authority for Token-2022 mints.
    /// PDA acting as mint and freeze authority for the resource mint.
    pub program_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [RESOURCE_MINT_SEED, &[resource_type]],
        bump,
        mint::token_program = token_program,
        mint::decimals = 0,
        mint::authority = program_authority,
        mint::freeze_authority = program_authority,
        extensions::metadata_pointer::authority = program_authority,
        extensions::metadata_pointer::metadata_address = resource_mint
    )]
    /// Token-2022 mint created for the requested resource type.
    pub resource_mint: InterfaceAccount<'info, Mint>,
    /// Token-2022 program used to initialize the mint.
    pub token_program: Program<'info, Token2022>,
    /// System program used to allocate the mint account.
    pub system_program: Program<'info, System>,
}

/// Accounts required for the authorized search CPI that mints resources to a player.
#[derive(Accounts)]
#[instruction(resource_type: u8)]
pub struct MintResourceToPlayer<'info> {
    /// Player receiving the resource tokens.
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        constraint = usize::from(resource_type) < RESOURCE_COUNT @ GameErrorCode::InvalidResourceTypeIndex
    )]
    /// Shared game config used to verify the canonical resource mint.
    pub game_config: Account<'info, GameConfig>,
    #[account(
        signer @ GameErrorCode::UnauthorizedResourceMintCaller,
        seeds = [PROGRAM_AUTHORITY_SEED],
        seeds::program = SEARCH_PROGRAM_ID,
        bump
    )]
    /// CHECK: this PDA is derived from the search program and must be signed via CPI.
    /// Search-program PDA proving the CPI caller is authorized.
    pub caller_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: this PDA is only used as the deterministic mint authority signer for Token-2022 CPI.
    /// Resource-manager PDA that signs the Token-2022 mint CPI.
    pub program_authority: UncheckedAccount<'info>,
    /// Resource mint to mint from.
    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = resource_mint,
        token::authority = player,
        token::token_program = token_program
    )]
    /// Player ATA that receives the minted resource tokens.
    pub player_resource_token_account: InterfaceAccount<'info, TokenAccount>,
    /// Token-2022 program used for minting.
    pub token_program: Program<'info, Token2022>,
}

/// Accounts required for the authorized crafting CPI that burns player resources.
#[derive(Accounts)]
#[instruction(resource_type: u8)]
pub struct BurnResourceFromPlayer<'info> {
    /// Player whose resource ATA is debited.
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        constraint = usize::from(resource_type) < RESOURCE_COUNT @ GameErrorCode::InvalidResourceTypeIndex
    )]
    /// Shared game config used to verify the canonical resource mint.
    pub game_config: Account<'info, GameConfig>,
    #[account(
        signer @ GameErrorCode::UnauthorizedResourceBurnCaller,
        seeds = [PROGRAM_AUTHORITY_SEED],
        seeds::program = CRAFTING_PROGRAM_ID,
        bump
    )]
    /// CHECK: this PDA is derived from the crafting program and must be signed via CPI.
    /// Crafting-program PDA proving the CPI caller is authorized.
    pub caller_authority: UncheckedAccount<'info>,
    /// Resource mint whose tokens will be burned.
    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = resource_mint,
        token::authority = player,
        token::token_program = token_program
    )]
    /// Player ATA that supplies the burned resource tokens.
    pub player_resource_token_account: InterfaceAccount<'info, TokenAccount>,
    /// Token-2022 program used for burning.
    pub token_program: Program<'info, Token2022>,
}

/// Singleton configuration account shared across the whole game.
#[account]
#[derive(InitSpace)]
pub struct GameConfig {
    /// Admin wallet allowed to perform one-time bootstrap actions.
    pub admin: Pubkey,
    /// Canonical resource mint PDAs ordered by `ResourceType`.
    pub resource_mints: [Pubkey; RESOURCE_COUNT],
    /// Configured reward mint used by marketplace redemptions.
    pub reward_token_mint: Pubkey,
    /// Reward amounts for each item type ordered by `ItemType`.
    pub item_prices: [u64; ITEM_COUNT],
    /// PDA bump for the `GameConfig` account.
    pub bump: u8,
}
