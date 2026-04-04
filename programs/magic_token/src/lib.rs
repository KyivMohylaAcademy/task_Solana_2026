use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, program::invoke_signed, system_instruction};
use anchor_spl::token_2022::{self, MintTo, Token2022};
use anchor_spl::token_2022_extensions::{metadata_pointer_initialize, MetadataPointerInitialize};
use anchor_spl::token_interface::{Mint as TokenMint, TokenAccount};
use game_common::{
    marketplace_id, GAME_CONFIG_SEED, MAGIC_AUTHORITY_SEED, MAGIC_MINT_SEED,
    MARKETPLACE_AUTHORITY_SEED,
};
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::state::Mint as SplMint;

declare_id!("D6TYLNDSrga9igvU5NwHwjgYtxyeTvLNPXGB9fF5p1PB");

#[program]
pub mod magic_token {
    use super::*;

    /// Creates the canonical MagicToken mint PDA.
    pub fn initialize_magic_token(
        ctx: Context<InitializeMagicToken>,
        _name: String,
        _symbol: String,
        _uri: String,
    ) -> Result<()> {
        require!(
            ctx.accounts.game_config.admin == ctx.accounts.admin.key(),
            ErrorCode::Unauthorized
        );
        require!(
            ctx.accounts.magic_mint.to_account_info().data_is_empty(),
            ErrorCode::MintAlreadyInitialized
        );
        let mint_len = ExtensionType::try_calculate_account_len::<SplMint>(&[ExtensionType::MetadataPointer])
            .expect("metadata pointer mint layout is valid");
        let lamports = Rent::get()?.minimum_balance(mint_len);
        let game_config_key = ctx.accounts.game_config.key();
        let mint_signer_seeds: &[&[u8]] = &[
            MAGIC_MINT_SEED,
            game_config_key.as_ref(),
            &[ctx.bumps.magic_mint],
        ];

        invoke_signed(
            &system_instruction::create_account(
                &ctx.accounts.admin.key(),
                &ctx.accounts.magic_mint.key(),
                lamports,
                mint_len as u64,
                &ctx.accounts.token_program.key(),
            ),
            &[
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.magic_mint.to_account_info(),
            ],
            &[mint_signer_seeds],
        )?;

        metadata_pointer_initialize(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MetadataPointerInitialize {
                    token_program_id: ctx.accounts.token_program.to_account_info(),
                    mint: ctx.accounts.magic_mint.to_account_info(),
                },
            ),
            Some(ctx.accounts.mint_authority.key()),
            Some(ctx.accounts.magic_mint.key()),
        )?;

        let initialize_mint_ix = spl_token_2022::instruction::initialize_mint2(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.magic_mint.key(),
            &ctx.accounts.mint_authority.key(),
            Some(&ctx.accounts.mint_authority.key()),
            0,
        )?;
        invoke(
            &initialize_mint_ix,
            &[
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.magic_mint.to_account_info(),
            ],
        )?;

        Ok(())
    }

    /// Mints MagicToken to the seller. Only the marketplace CPI path may call it.
    pub fn mint_reward(ctx: Context<MintReward>, amount: u64) -> Result<()> {
        require!(
            ctx.accounts.game_config.magic_token_mint == ctx.accounts.magic_mint.key(),
            ErrorCode::InvalidMint
        );
        let game_config_key = ctx.accounts.game_config.key();

        let expected_marketplace = Pubkey::find_program_address(
            &[MARKETPLACE_AUTHORITY_SEED, game_config_key.as_ref()],
            &marketplace_id(),
        )
        .0;
        require_keys_eq!(
            expected_marketplace,
            ctx.accounts.marketplace_authority.key(),
            ErrorCode::UnauthorizedCaller
        );

        let signer_seeds: &[&[u8]] = &[
            MAGIC_AUTHORITY_SEED,
            game_config_key.as_ref(),
            &[ctx.bumps.mint_authority],
        ];
        token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.magic_mint.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMagicToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID
    )]
    pub game_config: Account<'info, resource_manager::GameConfig>,
    /// CHECK: The handler derives and creates this PDA as the canonical MagicToken mint.
    #[account(mut, seeds = [MAGIC_MINT_SEED, game_config.key().as_ref()], bump)]
    pub magic_mint: UncheckedAccount<'info>,
    /// CHECK: PDA signer used as the only mint authority for MagicToken.
    #[account(seeds = [MAGIC_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintReward<'info> {
    pub marketplace_authority: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID
    )]
    pub game_config: Account<'info, resource_manager::GameConfig>,
    #[account(mut)]
    pub magic_mint: InterfaceAccount<'info, TokenMint>,
    #[account(mut)]
    pub destination: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: PDA signer derived from the game config and validated by seeds.
    #[account(seeds = [MAGIC_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Only the configured admin can initialize MagicToken.")]
    Unauthorized,
    #[msg("MagicToken mint already exists.")]
    MintAlreadyInitialized,
    #[msg("MagicToken mint in config does not match the provided mint.")]
    InvalidMint,
    #[msg("Only the marketplace CPI path may mint MagicToken.")]
    UnauthorizedCaller,
}
