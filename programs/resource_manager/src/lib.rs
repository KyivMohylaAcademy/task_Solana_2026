use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use spl_token_2022::instruction as token_instruction;

declare_id!("CScUCDipS5gMCz7uMD4bqpjCt2NXLhJHoBWiXPKj2bEc");

pub const RESOURCE_WOOD: u8 = 0;
pub const RESOURCE_IRON: u8 = 1;
pub const RESOURCE_GOLD: u8 = 2;
pub const RESOURCE_LEATHER: u8 = 3;
pub const RESOURCE_STONE: u8 = 4;
pub const RESOURCE_DIAMOND: u8 = 5;
pub const NUM_RESOURCES: usize = 6;

pub const RESOURCE_SYMBOLS: [&str; NUM_RESOURCES] = [
    "WOOD", "IRON", "GOLD", "LEATHER", "STONE", "DIAMOND",
];

#[program]
pub mod resource_manager {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;
        game_config.admin = ctx.accounts.admin.key();
        game_config.bump = ctx.bumps.game_config;
        msg!("ResourceManager: Initialized game config");
        Ok(())
    }

    pub fn create_resource_mint(ctx: Context<CreateResourceMint>, resource_id: u8) -> Result<()> {
        require!(
            (resource_id as usize) < NUM_RESOURCES,
            GameError::InvalidResourceId
        );

        let game_config = &mut ctx.accounts.game_config;
        require!(
            ctx.accounts.admin.key() == game_config.admin,
            GameError::Unauthorized
        );

        let idx = resource_id as usize;
        let resource_id_bytes = [resource_id];
        let mint_bump = ctx.bumps.resource_mint;
        let seeds: &[&[u8]] = &[b"resource_mint", resource_id_bytes.as_ref(), &[mint_bump]];
        let signer_seeds = &[seeds];

        let mint_key = ctx.accounts.resource_mint.key();
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
                ctx.accounts.resource_mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            signer_seeds,
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
            &[ctx.accounts.resource_mint.to_account_info()],
            signer_seeds,
        )?;

        game_config.resource_mints[resource_id as usize] = mint_key;

        msg!(
            "ResourceManager: Created resource mint {} ({})",
            RESOURCE_SYMBOLS[idx],
            mint_key
        );
        Ok(())
    }

    pub fn mint_resource(ctx: Context<MintResource>, resource_id: u8, amount: u64) -> Result<()> {
        require!(
            (resource_id as usize) < NUM_RESOURCES,
            GameError::InvalidResourceId
        );

        let authority_seeds: &[&[u8]] = &[b"mint_authority", &[ctx.bumps.mint_authority]];

        token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token_2022::MintTo {
                    mint: ctx.accounts.resource_mint.to_account_info(),
                    to: ctx.accounts.player_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[authority_seeds],
            ),
            amount,
        )?;

        msg!("ResourceManager: Minted {} of resource {}", amount, resource_id);
        Ok(())
    }

    pub fn burn_resource(ctx: Context<BurnResource>, _resource_id: u8, amount: u64) -> Result<()> {
        token_2022::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token_2022::Burn {
                    mint: ctx.accounts.resource_mint.to_account_info(),
                    from: ctx.accounts.player_token_account.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            amount,
        )?;

        msg!("ResourceManager: Burned {} tokens", amount);
        Ok(())
    }
}

#[account]
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; NUM_RESOURCES],
    pub bump: u8,
}

impl GameConfig {
    pub const LEN: usize = 8 + 32 + (32 * NUM_RESOURCES) + 1;
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = GameConfig::LEN, seeds = [b"game_config"], bump)]
    pub game_config: Account<'info, GameConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct CreateResourceMint<'info> {
    #[account(mut, seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,

    /// CHECK: Token-2022 mint PDA, created via invoke_signed
    #[account(mut, seeds = [b"resource_mint", &[resource_id]], bump)]
    pub resource_mint: AccountInfo<'info>,

    /// CHECK: PDA mint authority
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: AccountInfo<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: Token-2022 program
    #[account(address = spl_token_2022::id())]
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct MintResource<'info> {
    #[account(seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,

    #[account(mut, address = game_config.resource_mints[resource_id as usize])]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: PDA mint authority
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: AccountInfo<'info>,

    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct BurnResource<'info> {
    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,
    pub player: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[error_code]
pub enum GameError {
    #[msg("Invalid resource ID. Must be 0-5.")]
    InvalidResourceId,
    #[msg("Unauthorized access.")]
    Unauthorized,
    #[msg("Mint creation failed.")]
    MintCreationFailed,
}
