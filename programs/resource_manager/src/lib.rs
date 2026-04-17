use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

declare_id!("C9jeF5eivo4126iDkktjdGk7MEJqNwY9V2pFXMwQYMcy");

#[program]
pub mod resource_manager {
    use super::*;

    /// Ініціалізує GameConfig PDA
    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        let config = &mut ctx.accounts.game_config;
        config.admin = ctx.accounts.admin.key();
        config.bump = ctx.bumps.game_config;
        Ok(())
    }

    /// Реєструє мінт ресурсу в GameConfig
    pub fn register_resource(ctx: Context<RegisterResource>, index: u8) -> Result<()> {
        require!(index < 6, GameError::InvalidResourceIndex);
        let config = &mut ctx.accounts.game_config;
        config.resource_mints[index as usize] = ctx.accounts.mint.key();
        msg!("Registered resource {} : {}", index, ctx.accounts.mint.key());
        Ok(())
    }

pub fn mint_resource(ctx: Context<MintResource>, amount: u64) -> Result<()> {
    let bump = ctx.accounts.game_config.bump;
    let seeds: &[&[u8]] = &[b"game_config", &[bump]];
    let signer_seeds = &[seeds];

    anchor_spl::token_2022::mint_to(
        CpiContext::new_with_signer(
            anchor_spl::token_2022::ID,
            anchor_spl::token_2022::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.game_config.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;
    Ok(())
}

pub fn burn_resource(ctx: Context<BurnResource>, amount: u64) -> Result<()> {
    anchor_spl::token_2022::burn(
        CpiContext::new(
            anchor_spl::token_2022::ID,
            anchor_spl::token_2022::Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.player.to_account_info(),
            },
        ),
        amount,
    )?;
    Ok(())
}
}

#[account]
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; 6],
    pub magic_token_mint: Pubkey,
    pub bump: u8,
}

impl GameConfig {
    pub const LEN: usize = 8 + 32 + (32 * 6) + 32 + 1;
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = admin,
        space = GameConfig::LEN,
        seeds = [b"game_config"],
        bump
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterResource<'info> {
    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump,
        has_one = admin
    )]
    pub game_config: Account<'info, GameConfig>,

    pub mint: InterfaceAccount<'info, Mint>,

    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct MintResource<'info> {
    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: токен акаунт гравця
    #[account(mut)]
    pub player_token_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, anchor_spl::token_2022::Token2022>,
}

#[derive(Accounts)]
pub struct BurnResource<'info> {
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: токен акаунт гравця
    #[account(mut)]
    pub player_token_account: UncheckedAccount<'info>,

    pub player: Signer<'info>,

    pub token_program: Program<'info, anchor_spl::token_2022::Token2022>,
}

#[error_code]
pub enum GameError {
    #[msg("Невірний індекс ресурсу (0-5)")]
    InvalidResourceIndex,
}