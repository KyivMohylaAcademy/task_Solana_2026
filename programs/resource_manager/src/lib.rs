//! # resource_manager
//!
//! Manages the six SPL Token-2022 resource mints (WOOD, IRON, GOLD, LEATHER, STONE, DIAMOND).
//! All mints use the `MetadataPointer` and `TokenMetadata` extensions with `decimals = 0`.
//!
//! ## Authority model
//! - Mint authority for each resource mint is the `resource_authority` PDA of this program.
//! - `mint_from_search` requires a signer PDA derived as `["search_authority"]` under the
//!   search program ID – only the search program can produce this.
//! - `mint_from_crafting` and `burn_from_crafting` require a signer PDA derived as
//!   `["crafting_authority"]` under the crafting program ID.

use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::{
    extension::{
        metadata_pointer::instruction::initialize as metadata_pointer_initialize,
        ExtensionType,
    },
    instruction::initialize_mint2,
    state::Mint as SplMint,
};
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{
        Mint as InterfaceMint, TokenAccount as InterfaceTokenAccount, MintTo, BurnChecked,
        mint_to, burn_checked,
    },
};
use shared::{
    seeds::*,
    state::GameConfig,
    errors::GameError,
    items::ResourceKind,
};
use spl_token_2022::extension::StateWithExtensionsMut;
use solana_program::{
    program::invoke_signed,
    system_instruction,
};

declare_id!("F28jgR2vTiCi8PN9FW5B3v7JcBsu2NEPTJiX4KGxx2mj");

/// Search program ID – used to validate the search_authority PDA signer.
pub mod search_program {
    anchor_lang::declare_id!("9ZEk766xrSnSqJ4ke1vY9FhiGJwXZk37YK1ApBQaB6Pg");
}

/// Crafting program ID – used to validate the crafting_authority PDA signer.
pub mod crafting_program {
    anchor_lang::declare_id!("B2mXTz3cVrn3UubqVTKyqyEWh6qTiVcCjn1DQw8azB65");
}

#[program]
pub mod resource_manager {
    use super::*;

    /// Initialize the global GameConfig PDA. Must be called once by the admin before
    /// any other instructions. Sets item prices and the default 60-second cooldown.
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        item_prices: [u64; 4],
    ) -> Result<()> {
        let config = &mut ctx.accounts.game_config;
        config.admin = ctx.accounts.admin.key();
        config.resource_mints = [Pubkey::default(); 6];
        config.magic_token_mint = Pubkey::default();
        config.item_prices = item_prices;
        config.cooldown_seconds = 60;
        config.bump = ctx.bumps.game_config;
        Ok(())
    }

    /// Update item prices (admin only).
    pub fn update_item_prices(
        ctx: Context<AdminOnly>,
        item_prices: [u64; 4],
    ) -> Result<()> {
        let config = &mut ctx.accounts.game_config;
        require!(config.admin == ctx.accounts.admin.key(), GameError::AdminOnly);
        config.item_prices = item_prices;
        Ok(())
    }

    /// Update search cooldown in seconds (admin only, useful for testing).
    pub fn update_cooldown(
        ctx: Context<AdminOnly>,
        cooldown_seconds: i64,
    ) -> Result<()> {
        let config = &mut ctx.accounts.game_config;
        require!(config.admin == ctx.accounts.admin.key(), GameError::AdminOnly);
        config.cooldown_seconds = cooldown_seconds;
        Ok(())
    }

    /// Set the MagicToken mint address in GameConfig (called by magic_token program after init).
    pub fn set_magic_token_mint(
        ctx: Context<SetMagicTokenMint>,
        mint: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.game_config;
        require!(config.admin == ctx.accounts.admin.key(), GameError::AdminOnly);
        require!(
            config.magic_token_mint == Pubkey::default(),
            GameError::MagicMintAlreadySet
        );
        config.magic_token_mint = mint;
        Ok(())
    }

    /// Create one of the 6 resource mints with Token-2022 MetadataPointer + TokenMetadata
    /// extensions. Admin only. Stores the mint address in GameConfig.
    pub fn create_resource_mint(
        ctx: Context<CreateResourceMint>,
        kind: u8,
    ) -> Result<()> {
        let resource_kind = ResourceKind::from_u8(kind)
            .ok_or(GameError::InvalidResourceKind)?;

        let config = &mut ctx.accounts.game_config;
        require!(config.admin == ctx.accounts.admin.key(), GameError::AdminOnly);
        require!(
            config.resource_mints[kind as usize] == Pubkey::default(),
            GameError::MintAlreadyCreated
        );

        let authority_seeds: &[&[u8]] = &[
            RESOURCE_AUTHORITY_SEED,
            &[ctx.bumps.resource_authority],
        ];
        let signer_seeds = &[authority_seeds];

        // Allocate space for the mint with the two extensions
        let mint_key = ctx.accounts.mint.key();
        let space = ExtensionType::try_calculate_account_len::<SplMint>(
            &[ExtensionType::MetadataPointer, ExtensionType::TokenMetadata],
        ).map_err(|_| error!(GameError::Overflow))?;

        // The Token-2022 token metadata requires extra variable space for name/symbol/uri.
        // We add a generous fixed buffer here.
        let name = resource_kind.name().to_string();
        let symbol = resource_kind.symbol().to_string();
        let uri = String::new();
        let extra = 4 + name.len() + 4 + symbol.len() + 4 + uri.len() + 4; // TLV overhead
        let total_space = space + extra;

        // Fund and create the mint account
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(total_space);

        anchor_lang::solana_program::program::invoke(
            &system_instruction::create_account(
                ctx.accounts.admin.key,
                &mint_key,
                lamports,
                total_space as u64,
                &Token2022::id(),
            ),
            &[
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Initialize MetadataPointer extension (points to self)
        let init_metadata_ptr_ix = metadata_pointer_initialize(
            &Token2022::id(),
            &mint_key,
            Some(ctx.accounts.resource_authority.key()),
            Some(mint_key),
        ).map_err(|_| error!(GameError::Overflow))?;

        anchor_lang::solana_program::program::invoke(
            &init_metadata_ptr_ix,
            &[
                ctx.accounts.mint.to_account_info(),
            ],
        )?;

        // Initialize the mint (mint authority = resource_authority PDA)
        anchor_lang::solana_program::program::invoke(
            &initialize_mint2(
                &Token2022::id(),
                &mint_key,
                ctx.accounts.resource_authority.key,
                None,
                0,
            )?,
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;

        // Initialize TokenMetadata extension
        let init_metadata_ix = spl_token_2022::extension::metadata::instruction::initialize(
            &Token2022::id(),
            &mint_key,
            ctx.accounts.resource_authority.key,
            &mint_key,
            ctx.accounts.resource_authority.key,
            name,
            symbol,
            uri,
        );

        invoke_signed(
            &init_metadata_ix,
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.resource_authority.to_account_info(),
            ],
            signer_seeds,
        )?;

        config.resource_mints[kind as usize] = mint_key;
        Ok(())
    }

    /// Mint resource tokens to a player's ATA. Only callable via CPI from the search program
    /// (enforced by requiring the search_authority PDA as a signer).
    pub fn mint_from_search(
        ctx: Context<MintFromSearch>,
        kind: u8,
        amount: u64,
    ) -> Result<()> {
        ResourceKind::from_u8(kind).ok_or(GameError::InvalidResourceKind)?;

        let authority_bump = ctx.bumps.resource_authority;
        let seeds: &[&[u8]] = &[RESOURCE_AUTHORITY_SEED, &[authority_bump]];
        let signer_seeds = &[seeds];

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.player_ata.to_account_info(),
                    authority: ctx.accounts.resource_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        Ok(())
    }

    /// Mint resource tokens to a player's ATA. Only callable via CPI from the crafting program.
    pub fn mint_from_crafting(
        ctx: Context<MintFromCrafting>,
        kind: u8,
        amount: u64,
    ) -> Result<()> {
        ResourceKind::from_u8(kind).ok_or(GameError::InvalidResourceKind)?;

        let authority_bump = ctx.bumps.resource_authority;
        let seeds: &[&[u8]] = &[RESOURCE_AUTHORITY_SEED, &[authority_bump]];
        let signer_seeds = &[seeds];

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.player_ata.to_account_info(),
                    authority: ctx.accounts.resource_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        Ok(())
    }

    /// Burn resource tokens from a player's ATA. Only callable via CPI from the crafting program.
    /// The player must also be a signer (they own the tokens).
    pub fn burn_from_crafting(
        ctx: Context<BurnFromCrafting>,
        _kind: u8,
        amount: u64,
    ) -> Result<()> {
        burn_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                BurnChecked {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.player_ata.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            amount,
            0, // decimals = 0
        )?;
        Ok(())
    }
}

// ─── Account structs ──────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = GameConfig::LEN,
        seeds = [GAME_CONFIG_SEED],
        bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
    )]
    pub game_config: Account<'info, GameConfig>,
}

#[derive(Accounts)]
pub struct SetMagicTokenMint<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
    )]
    pub game_config: Account<'info, GameConfig>,
}

#[derive(Accounts)]
#[instruction(kind: u8)]
pub struct CreateResourceMint<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// The new mint account – must be pre-allocated as a new keypair.
    /// CHECK: We create and initialize it manually with Token-2022 extensions.
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    /// PDA that will be the mint authority for this resource.
    /// CHECK: Verified by seeds constraint.
    #[account(
        seeds = [RESOURCE_AUTHORITY_SEED],
        bump,
    )]
    pub resource_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(kind: u8, amount: u64)]
pub struct MintFromSearch<'info> {
    /// search_authority PDA from the search program. Verified via seeds::program.
    #[account(
        seeds = [SEARCH_AUTHORITY_SEED],
        bump,
        seeds::program = search_program::ID,
    )]
    pub search_authority: Signer<'info>,

    #[account(
        seeds = [RESOURCE_AUTHORITY_SEED],
        bump,
    )]
    /// CHECK: PDA used as mint authority.
    pub resource_authority: UncheckedAccount<'info>,

    /// CHECK: Caller is responsible for passing the correct mint for `kind`.
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Caller ensures correct player ATA.
    pub player_ata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
#[instruction(kind: u8, amount: u64)]
pub struct MintFromCrafting<'info> {
    #[account(
        seeds = [CRAFTING_AUTHORITY_SEED],
        bump,
        seeds::program = crafting_program::ID,
    )]
    pub crafting_authority: Signer<'info>,

    #[account(
        seeds = [RESOURCE_AUTHORITY_SEED],
        bump,
    )]
    /// CHECK: PDA used as mint authority.
    pub resource_authority: UncheckedAccount<'info>,

    /// CHECK: Caller ensures correct mint.
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Caller ensures correct player ATA.
    pub player_ata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
#[instruction(_kind: u8, amount: u64)]
pub struct BurnFromCrafting<'info> {
    #[account(
        seeds = [CRAFTING_AUTHORITY_SEED],
        bump,
        seeds::program = crafting_program::ID,
    )]
    pub crafting_authority: Signer<'info>,

    /// Player must sign to authorize burning from their own ATA.
    pub player: Signer<'info>,

    /// CHECK: Caller ensures correct mint.
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Caller ensures correct player ATA.
    pub player_ata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
}
