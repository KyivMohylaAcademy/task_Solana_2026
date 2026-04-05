//! Reward-token mint management for marketplace redemptions.

use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface};
use resource_manager::GameConfig;
use shared::{
    GameErrorCode, GAME_CONFIG_SEED, MAGIC_TOKEN_MINT_SEED, MARKETPLACE_PROGRAM_ID,
    PROGRAM_AUTHORITY_SEED, RESOURCE_MANAGER_PROGRAM_ID,
};

declare_id!("Bvw1CY1ZBu7jE2zmmKkWKe75LfoQvudwT11YxGYaLGW");

/// Owns the default reward mint and authorizes marketplace rewards.
#[program]
pub mod magic_token {
    use super::*;

    /// Creates the default Token-2022 reward mint.
    pub fn initialize_magic_token_mint(ctx: Context<InitializeMagicTokenMint>) -> Result<()> {
        shared::validate_bootstrap_config()?;

        require_keys_eq!(
            ctx.accounts.magic_token_mint.key(),
            ctx.accounts.game_config.reward_token_mint,
            GameErrorCode::MagicTokenMintAddressMismatch
        );

        Ok(())
    }

    /// Mints rewards to a player when invoked by the marketplace CPI path.
    pub fn mint_magic_to_player(ctx: Context<MintMagicToPlayer>, amount: u64) -> Result<()> {
        require!(amount > 0, GameErrorCode::InvalidTokenAmount);

        require_keys_eq!(
            ctx.accounts.magic_token_mint.key(),
            ctx.accounts.game_config.reward_token_mint,
            GameErrorCode::MagicTokenMintAddressMismatch
        );
        require!(
            ctx.accounts.caller_authority.is_signer,
            GameErrorCode::UnauthorizedMagicMintCaller
        );

        let authority_bump = ctx.bumps.program_authority;
        let signer_seeds: &[&[u8]] = &[PROGRAM_AUTHORITY_SEED, &[authority_bump]];
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.magic_token_mint.to_account_info(),
                    to: ctx.accounts.player_magic_token_account.to_account_info(),
                    authority: ctx.accounts.program_authority.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
        )?;

        Ok(())
    }
}

/// Accounts required to create the default reward mint.
#[derive(Accounts)]
pub struct InitializeMagicTokenMint<'info> {
    /// Bootstrap admin paying rent for the mint.
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump = game_config.bump,
        constraint = game_config.admin == admin.key() @ GameErrorCode::UnauthorizedAdmin
    )]
    /// Shared config account authorizing the admin and expected mint PDA.
    pub game_config: Account<'info, GameConfig>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: this PDA is only used as the deterministic mint/freeze authority for the Token-2022 mint.
    /// PDA acting as mint and freeze authority for the default reward mint.
    pub program_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [MAGIC_TOKEN_MINT_SEED],
        bump,
        mint::token_program = token_program,
        mint::decimals = 0,
        mint::authority = program_authority,
        mint::freeze_authority = program_authority,
        extensions::metadata_pointer::authority = program_authority,
        extensions::metadata_pointer::metadata_address = magic_token_mint
    )]
    /// Token-2022 mint account for the default reward token.
    pub magic_token_mint: InterfaceAccount<'info, Mint>,
    /// Token-2022 program used to initialize the mint.
    pub token_program: Program<'info, Token2022>,
    /// System program used to allocate the mint account.
    pub system_program: Program<'info, System>,
}

/// Accounts required for the authorized marketplace CPI that mints rewards.
#[derive(Accounts)]
pub struct MintMagicToPlayer<'info> {
    /// Player receiving the configured reward.
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump = game_config.bump
    )]
    /// Shared game config used to verify the configured reward mint.
    pub game_config: Account<'info, GameConfig>,
    #[account(
        signer @ GameErrorCode::UnauthorizedMagicMintCaller,
        seeds = [PROGRAM_AUTHORITY_SEED],
        seeds::program = MARKETPLACE_PROGRAM_ID,
        bump
    )]
    /// CHECK: this PDA is derived from the marketplace program and must be signed via CPI.
    /// Marketplace-program PDA proving the CPI caller is authorized.
    pub caller_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: this PDA is only used as the deterministic mint authority signer for Token-2022 CPI.
    /// Magic-token PDA that signs the reward mint CPI.
    pub program_authority: UncheckedAccount<'info>,
    /// Reward mint configured in `GameConfig`.
    #[account(mut)]
    pub magic_token_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        token::mint = magic_token_mint,
        token::authority = player,
        token::token_program = token_program
    )]
    /// Player ATA that receives the minted reward tokens.
    pub player_magic_token_account: InterfaceAccount<'info, TokenAccount>,
    /// Token program interface used for minting.
    pub token_program: Interface<'info, TokenInterface>,
}
