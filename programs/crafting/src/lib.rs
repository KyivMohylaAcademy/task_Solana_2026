//! Crafting program: burns resources via [`resource_manager`] then mints an item NFT via [`item_nft`] CPI.

use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use item_nft::cpi::{accounts::MintItem, mint_item};
use resource_manager::cpi::{accounts::BurnForCraft, burn_for_craft};
use shared::GameConfig;

declare_id!("HpszmFiwYo2K5qs2Gv5HauKSUvqPLBEXLcQUjJWFYrwE");

const CRAFT_SEED: &[u8] = b"craft";

fn recipe_amounts(item_type: u8) -> Result<[u64; 6]> {
    let mut a = [0u64; 6];
    match item_type {
        0 => {
            a[1] = 3;
            a[0] = 1;
            a[3] = 1;
        }
        1 => {
            a[0] = 2;
            a[2] = 1;
            a[5] = 1;
        }
        2 => {
            a[3] = 4;
            a[1] = 2;
            a[2] = 1;
        }
        3 => {
            a[1] = 4;
            a[2] = 2;
            a[5] = 2;
        }
        _ => return err!(CraftingError::BadItemType),
    }
    Ok(a)
}

#[program]
pub mod crafting {
    use super::*;

    /// Burns recipe resources then CPI-mints the NFT in `item_nft`.
    pub fn craft(ctx: Context<Craft>, item_type: u8) -> Result<()> {
        require!(item_type < 4, CraftingError::BadItemType);
        ctx.accounts.validate_ata()?;
        let amounts = recipe_amounts(item_type)?;
        let player_key = ctx.accounts.player.key();
        let bump = ctx.bumps.crafting_authority;
        let signer_seeds: &[&[u8]] = &[CRAFT_SEED, player_key.as_ref(), &[bump]];
        let signers: &[&[&[u8]]] = &[signer_seeds];

        burn_for_craft(
            CpiContext::new_with_signer(
                ctx.accounts.resource_manager_program.to_account_info(),
                BurnForCraft {
                    game_config: ctx.accounts.game_config.to_account_info(),
                    crafting_authority: ctx.accounts.crafting_authority.to_account_info(),
                    player: ctx.accounts.player.to_account_info(),
                    mint_wood: ctx.accounts.mint_wood.to_account_info(),
                    mint_iron: ctx.accounts.mint_iron.to_account_info(),
                    mint_gold: ctx.accounts.mint_gold.to_account_info(),
                    mint_leather: ctx.accounts.mint_leather.to_account_info(),
                    mint_stone: ctx.accounts.mint_stone.to_account_info(),
                    mint_diamond: ctx.accounts.mint_diamond.to_account_info(),
                    ata_wood: ctx.accounts.ata_wood.to_account_info(),
                    ata_iron: ctx.accounts.ata_iron.to_account_info(),
                    ata_gold: ctx.accounts.ata_gold.to_account_info(),
                    ata_leather: ctx.accounts.ata_leather.to_account_info(),
                    ata_stone: ctx.accounts.ata_stone.to_account_info(),
                    ata_diamond: ctx.accounts.ata_diamond.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                signers,
            ),
            amounts,
        )?;

        mint_item(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                MintItem {
                    payer: ctx.accounts.payer.to_account_info(),
                    player: ctx.accounts.player.to_account_info(),
                    crafting_authority: ctx.accounts.crafting_authority.to_account_info(),
                    game_config: ctx.accounts.game_config.to_account_info(),
                    item_metadata: ctx.accounts.item_metadata.to_account_info(),
                    item_mint: ctx.accounts.item_mint.to_account_info(),
                    mint_authority: ctx.accounts.mint_authority.to_account_info(),
                    player_item_ata: ctx.accounts.player_item_ata.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    associated_token_program: ctx
                        .accounts
                        .associated_token_program
                        .to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                signers,
            ),
            item_type,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Craft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub player: Signer<'info>,
    /// CHECK: Crafting PDA for `player`; constrained by seeds (CPI signs with `invoke_signed`).
    #[account(seeds = [CRAFT_SEED, player.key().as_ref()], bump)]
    pub crafting_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        owner = resource_manager::ID,
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager::ID
    )]
    pub game_config: Box<Account<'info, GameConfig>>,
    /// New mint keypair (signer); initialized by `item_nft::mint_item`.
    #[account(mut)]
    pub item_mint: Signer<'info>,
    /// CHECK: Item metadata PDA; `item_nft::mint_item` initializes when missing (seeds match expected PDA).
    #[account(
        mut,
        seeds = [b"item_meta", item_mint.key().as_ref()],
        bump,
        seeds::program = item_nft_program.key()
    )]
    pub item_metadata: UncheckedAccount<'info>,
    /// CHECK: Item mint authority PDA; constrained by seeds.
    #[account(
        seeds = [b"item_mint_auth", game_config.key().as_ref()],
        bump,
        seeds::program = item_nft_program.key()
    )]
    pub mint_authority: UncheckedAccount<'info>,
    /// CHECK: Player ATA for `item_mint`; must match `get_associated_token_address_with_program_id` (see `validate_ata`).
    #[account(mut)]
    pub player_item_ata: UncheckedAccount<'info>,
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
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: resource_manager program
    pub resource_manager_program: UncheckedAccount<'info>,
    /// CHECK: item_nft program
    pub item_nft_program: UncheckedAccount<'info>,
}

impl<'info> Craft<'info> {
    pub fn validate_ata(&self) -> Result<()> {
        let expected = get_associated_token_address_with_program_id(
            &self.player.key(),
            &self.item_mint.key(),
            &self.token_program.key(),
        );
        require_keys_eq!(expected, self.player_item_ata.key(), CraftingError::BadAta);
        Ok(())
    }
}

#[error_code]
pub enum CraftingError {
    #[msg("Invalid item type")]
    BadItemType,
    #[msg("Player item ATA address mismatch")]
    BadAta,
}
