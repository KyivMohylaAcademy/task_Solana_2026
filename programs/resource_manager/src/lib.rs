//! Resource manager: six SPL Token-2022 mints (WOOD..DIAMOND) with mint authority held by this program.
//! Minting from search and burning for crafting are only accepted when the corresponding program PDA signs.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::{invoke, invoke_signed},
    system_instruction, system_program,
    sysvar::rent::Rent,
};
use anchor_lang::AccountSerialize;
use anchor_spl::token_2022::{self, Burn, MintTo, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};
use shared::GameConfig;
use solana_sha256_hasher::hashv;

declare_id!("HAktvQC29ctNNZ1YHv3HTqVLGxsWE7UYLJcXBAByVGwP");

const SEARCH_SEED: &[u8] = b"search";
const CRAFT_SEED: &[u8] = b"craft";

#[program]
pub mod resource_manager {
    use super::*;

    /// Creates [`GameConfig`] and six resource mints (decimals 0).
    pub fn initialize(
        ctx: Context<Initialize>,
        item_prices: [u64; 4],
        search_program: Pubkey,
        crafting_program: Pubkey,
        item_nft_program: Pubkey,
        marketplace_program: Pubkey,
    ) -> Result<()> {
        prepare_game_config(
            &ctx.accounts.game_config,
            &ctx.accounts.payer,
            &ctx.accounts.system_program,
            ctx.program_id,
            ctx.bumps.game_config,
        )?;

        write_game_config(
            &ctx.accounts.game_config,
            GameConfig {
                admin: ctx.accounts.admin.key(),
                magic_token_mint: ctx.accounts.magic_token_mint.key(),
                item_prices,
                search_program,
                crafting_program,
                item_nft_program,
                marketplace_program,
                resource_mints: [
                    ctx.accounts.mint_wood.key(),
                    ctx.accounts.mint_iron.key(),
                    ctx.accounts.mint_gold.key(),
                    ctx.accounts.mint_leather.key(),
                    ctx.accounts.mint_stone.key(),
                    ctx.accounts.mint_diamond.key(),
                ],
                bump: ctx.bumps.game_config,
            },
        )?;

        Ok(())
    }

    /// Mints three random resource units (one each per draw) to the player ATAs (search program PDA must sign).
    pub fn mint_search_resources(ctx: Context<MintSearchResources>) -> Result<()> {
        require!(
            ctx.accounts.search_authority.is_signer,
            ResourceError::BadSearchAuthoritySigner
        );
        let cfg = &ctx.accounts.game_config;
        verify_search_pda(
            cfg,
            &ctx.accounts.player.key(),
            &ctx.accounts.search_authority.key(),
        )?;

        let clock = Clock::get()?;
        let mut amounts = [0u64; 6];
        for i in 0u8..3 {
            let h = hashv(&[
                SEARCH_SEED,
                ctx.accounts.player.key().as_ref(),
                &clock.unix_timestamp.to_le_bytes(),
                &clock.slot.to_le_bytes(),
                &[i],
            ]);
            let pick = (h.to_bytes()[0] as usize) % 6;
            amounts[pick] = amounts[pick].saturating_add(1);
        }

        let cfg_key = cfg.key();
        let bump = ctx.bumps.resource_authority;
        let signer_seeds: &[&[u8]] = &[b"resource_auth", cfg_key.as_ref(), &[bump]];
        let signers: &[&[&[u8]]] = &[signer_seeds];

        mint_if_nonzero(
            amounts[0],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_wood.to_account_info(),
            &ctx.accounts.ata_wood.to_account_info(),
            &ctx.accounts.resource_authority.to_account_info(),
            signers,
        )?;
        mint_if_nonzero(
            amounts[1],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_iron.to_account_info(),
            &ctx.accounts.ata_iron.to_account_info(),
            &ctx.accounts.resource_authority.to_account_info(),
            signers,
        )?;
        mint_if_nonzero(
            amounts[2],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_gold.to_account_info(),
            &ctx.accounts.ata_gold.to_account_info(),
            &ctx.accounts.resource_authority.to_account_info(),
            signers,
        )?;
        mint_if_nonzero(
            amounts[3],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_leather.to_account_info(),
            &ctx.accounts.ata_leather.to_account_info(),
            &ctx.accounts.resource_authority.to_account_info(),
            signers,
        )?;
        mint_if_nonzero(
            amounts[4],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_stone.to_account_info(),
            &ctx.accounts.ata_stone.to_account_info(),
            &ctx.accounts.resource_authority.to_account_info(),
            signers,
        )?;
        mint_if_nonzero(
            amounts[5],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_diamond.to_account_info(),
            &ctx.accounts.ata_diamond.to_account_info(),
            &ctx.accounts.resource_authority.to_account_info(),
            signers,
        )?;

        Ok(())
    }

    /// Burns resources for crafting; crafting program PDA must sign and the player must sign as ATA owner.
    pub fn burn_for_craft(ctx: Context<BurnForCraft>, amounts: [u64; 6]) -> Result<()> {
        require!(
            ctx.accounts.crafting_authority.is_signer,
            ResourceError::BadCraftAuthoritySigner
        );
        let cfg = &ctx.accounts.game_config;
        verify_craft_pda(
            cfg,
            &ctx.accounts.player.key(),
            &ctx.accounts.crafting_authority.key(),
        )?;

        burn_if_nonzero(
            amounts[0],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_wood.to_account_info(),
            &ctx.accounts.ata_wood.to_account_info(),
            &ctx.accounts.player.to_account_info(),
        )?;
        burn_if_nonzero(
            amounts[1],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_iron.to_account_info(),
            &ctx.accounts.ata_iron.to_account_info(),
            &ctx.accounts.player.to_account_info(),
        )?;
        burn_if_nonzero(
            amounts[2],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_gold.to_account_info(),
            &ctx.accounts.ata_gold.to_account_info(),
            &ctx.accounts.player.to_account_info(),
        )?;
        burn_if_nonzero(
            amounts[3],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_leather.to_account_info(),
            &ctx.accounts.ata_leather.to_account_info(),
            &ctx.accounts.player.to_account_info(),
        )?;
        burn_if_nonzero(
            amounts[4],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_stone.to_account_info(),
            &ctx.accounts.ata_stone.to_account_info(),
            &ctx.accounts.player.to_account_info(),
        )?;
        burn_if_nonzero(
            amounts[5],
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.mint_diamond.to_account_info(),
            &ctx.accounts.ata_diamond.to_account_info(),
            &ctx.accounts.player.to_account_info(),
        )?;

        Ok(())
    }
}

fn mint_if_nonzero<'info>(
    amount: u64,
    token_program: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    signers: &[&[&[u8]]],
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    let cpi = CpiContext::new_with_signer(
        token_program.clone(),
        MintTo {
            mint: mint.clone(),
            to: to.clone(),
            authority: authority.clone(),
        },
        signers,
    );
    token_2022::mint_to(cpi, amount)
}

fn burn_if_nonzero<'info>(
    amount: u64,
    token_program: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    from: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    let cpi = CpiContext::new(
        token_program.clone(),
        Burn {
            mint: mint.clone(),
            from: from.clone(),
            authority: authority.clone(),
        },
    );
    token_2022::burn(cpi, amount)
}

fn verify_search_pda(cfg: &GameConfig, player: &Pubkey, authority: &Pubkey) -> Result<()> {
    let (expected, _) =
        Pubkey::find_program_address(&[SEARCH_SEED, player.as_ref()], &cfg.search_program);
    require_keys_eq!(expected, *authority, ResourceError::BadSearchAuthority);
    Ok(())
}

fn verify_craft_pda(cfg: &GameConfig, player: &Pubkey, authority: &Pubkey) -> Result<()> {
    let (expected, _) =
        Pubkey::find_program_address(&[CRAFT_SEED, player.as_ref()], &cfg.crafting_program);
    require_keys_eq!(expected, *authority, ResourceError::BadCraftAuthority);
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub admin: Signer<'info>,
    /// CHECK: Magic token mint (created by `magic_token` program).
    pub magic_token_mint: UncheckedAccount<'info>,
    /// CHECK: PDA derived inside `prepare_game_config`; we allocate/assign it and then populate in handler.
    #[account(mut, seeds = [b"game_config"], bump)]
    pub game_config: UncheckedAccount<'info>,
    /// CHECK: PDA authority for all resource mints
    #[account(
        seeds = [b"resource_auth", game_config.key().as_ref()],
        bump
    )]
    pub resource_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = resource_authority,
        mint::freeze_authority = resource_authority,
    )]
    pub mint_wood: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = resource_authority,
        mint::freeze_authority = resource_authority,
    )]
    pub mint_iron: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = resource_authority,
        mint::freeze_authority = resource_authority,
    )]
    pub mint_gold: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = resource_authority,
        mint::freeze_authority = resource_authority,
    )]
    pub mint_leather: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = resource_authority,
        mint::freeze_authority = resource_authority,
    )]
    pub mint_stone: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = resource_authority,
        mint::freeze_authority = resource_authority,
    )]
    pub mint_diamond: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintSearchResources<'info> {
    #[account(mut, owner = crate::ID, seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Box<Account<'info, GameConfig>>,
    /// Search program PDA for `player`; must sign via `invoke_signed` in `search` CPI.
    pub search_authority: Signer<'info>,
    /// CHECK: player wallet (used for PDA derivation)
    pub player: UncheckedAccount<'info>,
    /// CHECK: Resource mint authority PDA; address fixed by `#[account(seeds = ...)]`.
    #[account(seeds = [b"resource_auth", game_config.key().as_ref()], bump)]
    pub resource_authority: UncheckedAccount<'info>,
    #[account(mut, address = game_config.resource_mints[0])]
    pub mint_wood: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[1])]
    pub mint_iron: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[2])]
    pub mint_gold: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[3])]
    pub mint_leather: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[4])]
    pub mint_stone: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[5])]
    pub mint_diamond: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub ata_wood: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_iron: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_gold: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_leather: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_stone: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_diamond: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct BurnForCraft<'info> {
    #[account(mut, owner = crate::ID, seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Box<Account<'info, GameConfig>>,
    /// Crafting program PDA for `player`; must sign via `invoke_signed` in `crafting` CPI.
    pub crafting_authority: Signer<'info>,
    pub player: Signer<'info>,
    #[account(mut, address = game_config.resource_mints[0])]
    pub mint_wood: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[1])]
    pub mint_iron: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[2])]
    pub mint_gold: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[3])]
    pub mint_leather: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[4])]
    pub mint_stone: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, address = game_config.resource_mints[5])]
    pub mint_diamond: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub ata_wood: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_iron: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_gold: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_leather: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_stone: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub ata_diamond: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Program<'info, Token2022>,
}

#[error_code]
pub enum ResourceError {
    #[msg("Invalid search authority PDA")]
    BadSearchAuthority,
    #[msg("Search authority must sign")]
    BadSearchAuthoritySigner,
    #[msg("Invalid crafting authority PDA")]
    BadCraftAuthority,
    #[msg("Crafting authority must sign")]
    BadCraftAuthoritySigner,
    #[msg("Game config account owned by unexpected program")]
    BadGameConfigOwner,
}

fn prepare_game_config<'info>(
    game_config: &UncheckedAccount<'info>,
    payer: &Signer<'info>,
    system_program: &Program<'info, System>,
    program_id: &Pubkey,
    bump: u8,
) -> Result<()> {
    let account_info = game_config.to_account_info();
    let payer_info = payer.to_account_info();
    let system_info = system_program.to_account_info();
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(GameConfig::LEN);

    if account_info.lamports() < required_lamports {
        invoke(
            &system_instruction::transfer(
                payer_info.key,
                account_info.key,
                required_lamports.saturating_sub(account_info.lamports()),
            ),
            &[
                payer_info.clone(),
                account_info.clone(),
                system_info.clone(),
            ],
        )?;
    }

    let seeds: &[&[u8]] = &[b"game_config", &[bump]];

    if account_info.owner == program_id {
        if account_info.data_len() != GameConfig::LEN {
            account_info.resize(GameConfig::LEN)?;
        }
    } else if account_info.owner == &system_program::ID || account_info.data_len() == 0 {
        invoke_signed(
            &system_instruction::allocate(account_info.key, GameConfig::LEN as u64),
            &[account_info.clone(), system_info.clone()],
            &[seeds],
        )?;
        invoke_signed(
            &system_instruction::assign(account_info.key, program_id),
            &[account_info.clone(), system_info.clone()],
            &[seeds],
        )?;
    } else if account_info.owner != program_id {
        return err!(ResourceError::BadGameConfigOwner);
    }

    Ok(())
}

fn write_game_config(game_config: &UncheckedAccount, value: GameConfig) -> Result<()> {
    let account_info = game_config.to_account_info();
    let mut data = account_info.try_borrow_mut_data()?;
    data.fill(0);
    value.try_serialize(&mut &mut data[..])?;
    Ok(())
}
