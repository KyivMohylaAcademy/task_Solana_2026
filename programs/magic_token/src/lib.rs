//! MagicToken program: SPL Token-2022 mint with authority restricted to this program.
//! Mint to sellers and burns from buyers happen only via CPI from the marketplace program PDA.

use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Burn, MintTo, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};

declare_id!("A9GEReews3Hp9RQsiiM3Q4Tfpvd9WKofzdgcW1zKg2Bc");

fn verify_marketplace_pda(cfg: &MagicTokenConfig, marketplace_authority: &Pubkey) -> Result<()> {
    let (expected, _) = Pubkey::find_program_address(&[MARKET_EXEC_SEED], &cfg.marketplace_program);
    require_keys_eq!(
        expected,
        *marketplace_authority,
        MagicTokenError::BadMarketplace
    );
    Ok(())
}

pub const MARKET_EXEC_SEED: &[u8] = b"market_exec";

#[account]
pub struct MagicTokenConfig {
    pub admin: Pubkey,
    pub marketplace_program: Pubkey,
    pub bump: u8,
}

impl MagicTokenConfig {
    pub const LEN: usize = 8 + 32 + 32 + 1;
}

#[program]
pub mod magic_token {
    use super::*;

    /// Initializes the Magic token mint and config. Call once from deploy/admin.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.config.admin = ctx.accounts.admin.key();
        ctx.accounts.config.marketplace_program = ctx.accounts.marketplace_program.key();
        ctx.accounts.config.bump = ctx.bumps.config;
        Ok(())
    }

    /// Mints Magic tokens to a seller ATA. Callable only as CPI from the marketplace PDA signer.
    pub fn mint_to_seller(ctx: Context<MintToSeller>, amount: u64) -> Result<()> {
        require!(amount > 0, MagicTokenError::InvalidAmount);
        require!(
            ctx.accounts.marketplace_authority.is_signer,
            MagicTokenError::BadMarketplaceSigner
        );
        crate::verify_marketplace_pda(
            &ctx.accounts.config,
            &ctx.accounts.marketplace_authority.key(),
        )?;
        let bump = ctx.bumps.mint_authority;
        let seeds: &[&[u8]] = &[b"magic_auth", &[bump]];
        let signer = &[seeds];

        let cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.seller_ata.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer,
        );
        token_2022::mint_to(cpi, amount)?;
        Ok(())
    }

    /// Burns Magic tokens from the buyer ATA during a purchase. Buyer must sign as token owner.
    pub fn burn_from_buyer(ctx: Context<BurnFromBuyer>, amount: u64) -> Result<()> {
        require!(amount > 0, MagicTokenError::InvalidAmount);
        require!(
            ctx.accounts.marketplace_authority.is_signer,
            MagicTokenError::BadMarketplaceSigner
        );
        crate::verify_marketplace_pda(
            &ctx.accounts.config,
            &ctx.accounts.marketplace_authority.key(),
        )?;
        require_keys_eq!(
            ctx.accounts.buyer.key(),
            ctx.accounts.buyer_ata.owner,
            MagicTokenError::InvalidBuyer
        );
        let cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.buyer_ata.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        );
        token_2022::burn(cpi, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub admin: Signer<'info>,
    /// CHECK: marketplace program id stored for PDA checks on client; mint CPI verifies signer.
    pub marketplace_program: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        space = MagicTokenConfig::LEN,
        seeds = [b"magic_config"],
        bump
    )]
    pub config: Account<'info, MagicTokenConfig>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 9,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    /// CHECK: PDA mint authority
    #[account(seeds = [b"magic_auth"], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintToSeller<'info> {
    #[account(
        seeds = [b"magic_config"],
        bump = config.bump,
        has_one = marketplace_program
    )]
    pub config: Account<'info, MagicTokenConfig>,
    /// CHECK: Marketplace executor PDA (`MARKET_EXEC_SEED`); runtime-checked via `verify_marketplace_pda` and must sign.
    #[account(mut)]
    pub marketplace_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub seller_ata: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: mint authority PDA
    #[account(seeds = [b"magic_auth"], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    /// CHECK: must match config
    pub marketplace_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct BurnFromBuyer<'info> {
    #[account(
        seeds = [b"magic_config"],
        bump = config.bump,
        has_one = marketplace_program
    )]
    pub config: Account<'info, MagicTokenConfig>,
    /// CHECK: Marketplace executor PDA (`MARKET_EXEC_SEED`); runtime-checked via `verify_marketplace_pda` and must sign.
    #[account(mut)]
    pub marketplace_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub buyer_ata: InterfaceAccount<'info, TokenAccount>,
    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token2022>,
    /// CHECK: must match config
    pub marketplace_program: UncheckedAccount<'info>,
}

#[error_code]
pub enum MagicTokenError {
    #[msg("Amount must be positive")]
    InvalidAmount,
    #[msg("Buyer does not own the token account")]
    InvalidBuyer,
    #[msg("Marketplace program mismatch")]
    BadMarketplace,
    #[msg("Marketplace authority must sign")]
    BadMarketplaceSigner,
}
