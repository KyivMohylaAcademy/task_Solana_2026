//! # magic_token
//!
//! Manages the MagicToken SPL Token-2022 mint. This is the in-game currency awarded
//! when players sell crafted items on the Marketplace.
//!
//! ## Authority model
//! - Mint authority = `magic_authority` PDA of this program.
//! - `mint_to_player` requires a `marketplace_authority` PDA signer derived under the
//!   marketplace program ID – only the marketplace program can invoke this.

use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint as InterfaceMint, TokenAccount as InterfaceTokenAccount, MintTo, mint_to},
};
use shared::{seeds::*, errors::GameError};

declare_id!("HfLuv435urC8rxobkUe89f2cEYAKFxPKuwQfuDAZzrzT");

/// Marketplace program ID for authority validation.
pub mod marketplace_program {
    anchor_lang::declare_id!("8FCw32yjvmK8po3yjH3U6p4ZNSzm7H7iCWiwjR6JHkzx");
}

/// Resource manager program ID for linking the mint into GameConfig.
pub mod resource_manager_program {
    anchor_lang::declare_id!("F28jgR2vTiCi8PN9FW5B3v7JcBsu2NEPTJiX4KGxx2mj");
}

#[program]
pub mod magic_token {
    use super::*;

    /// Create the MagicToken Token-2022 mint. Admin only. The mint address is a PDA so
    /// it is deterministic. After this instruction the admin should call
    /// `resource_manager::set_magic_token_mint` to register it in GameConfig.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // mint_authority = magic_authority PDA, already set via anchor init constraints
        let config = &mut ctx.accounts.magic_config;
        config.admin = ctx.accounts.admin.key();
        config.mint = ctx.accounts.magic_mint.key();
        config.bump = ctx.bumps.magic_config;
        Ok(())
    }

    /// Mint MagicToken to a player's ATA. Only callable via CPI from the marketplace program.
    pub fn mint_to_player(ctx: Context<MintToPlayer>, amount: u64) -> Result<()> {
        let authority_bump = ctx.bumps.magic_authority;
        let seeds: &[&[u8]] = &[MAGIC_AUTHORITY_SEED, &[authority_bump]];
        let signer_seeds = &[seeds];

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.magic_mint.to_account_info(),
                    to: ctx.accounts.player_ata.to_account_info(),
                    authority: ctx.accounts.magic_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        Ok(())
    }
}

// ─── State ────────────────────────────────────────────────────────────────────

#[account]
pub struct MagicTokenConfig {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

impl MagicTokenConfig {
    pub const LEN: usize = 8 + 32 + 32 + 1;
}

// ─── Account structs ──────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Token-2022 mint with magic_authority as mint authority.
    #[account(
        init,
        payer = admin,
        mint::decimals = 6,
        mint::authority = magic_authority,
        mint::token_program = token_program,
        seeds = [MAGIC_MINT_SEED],
        bump,
    )]
    pub magic_mint: InterfaceAccount<'info, InterfaceMint>,

    /// PDA that acts as the mint authority.
    /// CHECK: Verified by seeds.
    #[account(
        seeds = [MAGIC_AUTHORITY_SEED],
        bump,
    )]
    pub magic_authority: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin,
        space = MagicTokenConfig::LEN,
        seeds = [b"magic_config"],
        bump,
    )]
    pub magic_config: Account<'info, MagicTokenConfig>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintToPlayer<'info> {
    /// marketplace_authority PDA from the marketplace program.
    #[account(
        seeds = [MARKETPLACE_AUTHORITY_SEED],
        bump,
        seeds::program = marketplace_program::ID,
    )]
    pub marketplace_authority: Signer<'info>,

    /// CHECK: PDA used as mint authority, verified by seeds.
    #[account(
        seeds = [MAGIC_AUTHORITY_SEED],
        bump,
    )]
    pub magic_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [MAGIC_MINT_SEED],
        bump,
        mint::token_program = token_program,
    )]
    pub magic_mint: InterfaceAccount<'info, InterfaceMint>,

    #[account(mut)]
    /// CHECK: Caller ensures this is the player's valid MagicToken ATA.
    pub player_ata: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
}
