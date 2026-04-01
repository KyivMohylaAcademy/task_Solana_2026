//! Item NFT program: mints Token-2022 NFTs (decimals 0, supply 1) for crafted items.

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::{self, MintTo, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};
use shared::GameConfig;

declare_id!("7HRBGyd8KHHA8kUwGWZpcD3B3gvNv9mSu3gUzoGVKwsx");

#[account]
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

impl ItemMetadata {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 1;
}

const CRAFT_SEED: &[u8] = b"craft";

mod resource_manager_program {
    use anchor_lang::prelude::declare_id;

    declare_id!("HAktvQC29ctNNZ1YHv3HTqVLGxsWE7UYLJcXBAByVGwP");
}

#[program]
pub mod item_nft {
    use super::*;

    /// Mints a new item NFT with supply 1 to the player. Only the crafting PDA may invoke.
    pub fn mint_item(ctx: Context<MintItem>, item_type: u8) -> Result<()> {
        require!(item_type < 4, ItemNftError::BadItemType);
        require!(
            ctx.accounts.crafting_authority.is_signer,
            ItemNftError::BadCraftAuthoritySigner
        );
        let cfg = &ctx.accounts.game_config;
        verify_craft_pda(
            cfg,
            &ctx.accounts.player.key(),
            &ctx.accounts.crafting_authority.key(),
        )?;

        let cfg_key = cfg.key();
        let bump = ctx.bumps.mint_authority;
        let signer_seeds: &[&[u8]] = &[b"item_mint_auth", cfg_key.as_ref(), &[bump]];
        let signers: &[&[&[u8]]] = &[signer_seeds];

        let mint_cpi = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.item_mint.to_account_info(),
                to: ctx.accounts.player_item_ata.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signers,
        );
        token_2022::mint_to(mint_cpi, 1)?;

        let meta = &mut ctx.accounts.item_metadata;
        meta.item_type = item_type;
        meta.owner = ctx.accounts.player.key();
        meta.mint = ctx.accounts.item_mint.key();
        meta.bump = ctx.bumps.item_metadata;

        Ok(())
    }
}

fn verify_craft_pda(cfg: &GameConfig, player: &Pubkey, authority: &Pubkey) -> Result<()> {
    let (expected, _) =
        Pubkey::find_program_address(&[CRAFT_SEED, player.as_ref()], &cfg.crafting_program);
    require_keys_eq!(expected, *authority, ItemNftError::BadCraftAuthority);
    Ok(())
}

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct MintItem<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub player: Signer<'info>,
    /// Crafting program PDA for `player`; must sign via `invoke_signed` in `crafting` CPI.
    pub crafting_authority: Signer<'info>,
    #[account(
        mut,
        owner = resource_manager_program::ID,
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager_program::ID
    )]
    pub game_config: Account<'info, GameConfig>,
    #[account(
        init,
        payer = payer,
        space = ItemMetadata::LEN,
        seeds = [b"item_meta", item_mint.key().as_ref()],
        bump
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
    )]
    pub item_mint: InterfaceAccount<'info, Mint>,
    /// CHECK: Mint authority PDA; constrained by seeds.
    #[account(
        seeds = [b"item_mint_auth", game_config.key().as_ref()],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        associated_token::mint = item_mint,
        associated_token::authority = player,
        associated_token::token_program = token_program,
    )]
    pub player_item_ata: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[error_code]
pub enum ItemNftError {
    #[msg("Invalid item type")]
    BadItemType,
    #[msg("Invalid crafting authority PDA")]
    BadCraftAuthority,
    #[msg("Crafting authority must sign")]
    BadCraftAuthoritySigner,
}
