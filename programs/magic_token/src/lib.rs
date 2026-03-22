use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use spl_token_2022::instruction as token_instruction;

declare_id!("3LgzDM2mxShb8Bzp5JFLfboz6KXDMeQKtNwRW7Zf4aqd");

#[program]
pub mod magic_token {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.magic_token_config;
        config.admin = ctx.accounts.admin.key();
        config.bump = ctx.bumps.magic_token_config;

        let mint_key = ctx.accounts.magic_token_mint.key();
        config.mint = mint_key;

        let mint_bump = ctx.bumps.magic_token_mint;
        let mint_seeds: &[&[u8]] = &[b"magic_token_mint", &[mint_bump]];
        let mint_signer_seeds = &[mint_seeds];

        let authority_key = ctx.accounts.mint_authority.key();

        let space = 82usize;
        let lamports = Rent::get()?.minimum_balance(space);
        let create_ix = anchor_lang::solana_program::system_instruction::create_account(
            &ctx.accounts.admin.key(),
            &mint_key,
            lamports,
            space as u64,
            &spl_token_2022::id(),
        );
        invoke_signed(
            &create_ix,
            &[
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.magic_token_mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            mint_signer_seeds,
        )?;

        let init_mint_ix = token_instruction::initialize_mint2(
            &spl_token_2022::id(),
            &mint_key,
            &authority_key,
            None,
            0,
        )?;
        invoke_signed(
            &init_mint_ix,
            &[ctx.accounts.magic_token_mint.to_account_info()],
            mint_signer_seeds,
        )?;

        msg!("MagicToken: Initialized mint {}", mint_key);
        Ok(())
    }

    pub fn mint_magic_token(ctx: Context<MintMagicToken>, amount: u64) -> Result<()> {
        let authority_seeds: &[&[u8]] =
            &[b"magic_mint_authority", &[ctx.bumps.mint_authority]];

        token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token_2022::MintTo {
                    mint: ctx.accounts.magic_token_mint.to_account_info(),
                    to: ctx.accounts.player_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[authority_seeds],
            ),
            amount,
        )?;

        msg!("MagicToken: Minted {} tokens", amount);
        Ok(())
    }
}

#[account]
pub struct MagicTokenConfig {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

impl MagicTokenConfig {
    pub const LEN: usize = 8 + 32 + 32 + 1;
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = MagicTokenConfig::LEN, seeds = [b"magic_token_config"], bump)]
    pub magic_token_config: Account<'info, MagicTokenConfig>,

    /// CHECK: Token-2022 mint PDA, created via invoke_signed
    #[account(mut, seeds = [b"magic_token_mint"], bump)]
    pub magic_token_mint: AccountInfo<'info>,

    /// CHECK: PDA mint authority
    #[account(seeds = [b"magic_mint_authority"], bump)]
    pub mint_authority: AccountInfo<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: Token-2022 program
    #[account(address = spl_token_2022::id())]
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct MintMagicToken<'info> {
    #[account(seeds = [b"magic_token_config"], bump = magic_token_config.bump)]
    pub magic_token_config: Account<'info, MagicTokenConfig>,

    #[account(mut, address = magic_token_config.mint)]
    pub magic_token_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: PDA mint authority
    #[account(seeds = [b"magic_mint_authority"], bump)]
    pub mint_authority: AccountInfo<'info>,

    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[error_code]
pub enum MagicTokenError {
    #[msg("Mint creation failed.")]
    MintCreationFailed,
}
