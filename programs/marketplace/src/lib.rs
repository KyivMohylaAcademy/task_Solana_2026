//! Marketplace redemption logic that burns crafted NFTs for configured rewards.

use anchor_lang::prelude::*;
use anchor_spl::associated_token::{
    create_idempotent, get_associated_token_address_with_program_id, AssociatedToken, Create,
};
use anchor_spl::token::{Mint as SplMint, Token, TokenAccount};
use anchor_spl::token_interface::{Mint, TokenInterface};
use item_nft::{program::ItemNft, ItemMetadata};
use magic_token::program::MagicToken;
use resource_manager::GameConfig;
use shared::{
    GameErrorCode, ItemType, GAME_CONFIG_SEED, ITEM_COUNT, ITEM_NFT_PROGRAM_ID,
    MAGIC_TOKEN_PROGRAM_ID, PROGRAM_AUTHORITY_SEED, RESOURCE_MANAGER_PROGRAM_ID,
    TOKEN_METADATA_PROGRAM_ID,
};

declare_id!("3cPgZBSjpvcuD5FmhGQfCSBFXnz3ZMs573u8UDszgpeW");

/// Redeems crafted NFTs into configured reward tokens through coordinated CPIs.
#[program]
pub mod marketplace {
    use super::*;

    /// Verifies shared bootstrap invariants for the marketplace program.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        shared::validate_bootstrap_config()?;
        msg!("marketplace bootstrap ready");
        Ok(())
    }

    /// Burns a crafted item NFT and mints the configured reward payout.
    pub fn redeem_item_for_magic(ctx: Context<RedeemItemForMagic>, item_type: u8) -> Result<()> {
        shared::validate_bootstrap_config()?;

        let item_type = ItemType::from_index(usize::from(item_type))?;
        require_keys_eq!(
            ctx.accounts.mint.key(),
            ctx.accounts.item_metadata.mint,
            GameErrorCode::ItemMetadataMintMismatch
        );
        require_eq!(
            ctx.accounts.item_metadata.item_type,
            item_type as u8,
            GameErrorCode::ItemMetadataTypeMismatch
        );
        require!(
            ctx.accounts.owner_item_token_account.amount == 1,
            GameErrorCode::InvalidItemTokenAccount
        );
        require_keys_eq!(
            ctx.accounts.magic_token_mint.key(),
            ctx.accounts.game_config.reward_token_mint,
            GameErrorCode::MagicTokenMintAddressMismatch
        );

        let expected_magic_token_account = get_associated_token_address_with_program_id(
            &ctx.accounts.owner.key(),
            &ctx.accounts.magic_token_mint.key(),
            &ctx.accounts.magic_token_token_program.key(),
        );
        require_keys_eq!(
            ctx.accounts.player_magic_token_account.key(),
            expected_magic_token_account,
            GameErrorCode::InvalidMagicTokenAccount
        );

        create_idempotent(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            Create {
                payer: ctx.accounts.owner.to_account_info(),
                associated_token: ctx.accounts.player_magic_token_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
                mint: ctx.accounts.magic_token_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.magic_token_token_program.to_account_info(),
            },
        ))?;

        let authority_bump = ctx.bumps.marketplace_authority;
        let signer_seeds: &[&[u8]] = &[PROGRAM_AUTHORITY_SEED, &[authority_bump]];

        item_nft::cpi::burn_item_nft(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                item_nft::cpi::accounts::BurnItemNft {
                    owner: ctx.accounts.owner.to_account_info(),
                    game_config: ctx.accounts.game_config.to_account_info(),
                    caller_authority: ctx.accounts.marketplace_authority.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    item_metadata: ctx.accounts.item_metadata.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    master_edition: ctx.accounts.master_edition.to_account_info(),
                    owner_item_token_account: ctx
                        .accounts
                        .owner_item_token_account
                        .to_account_info(),
                    token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
                    token_program: ctx.accounts.item_token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    sysvar_instructions: ctx.accounts.sysvar_instructions.to_account_info(),
                },
                &[signer_seeds],
            ),
            item_type as u8,
        )?;

        magic_token::cpi::mint_magic_to_player(
            CpiContext::new_with_signer(
                ctx.accounts.magic_token_program.to_account_info(),
                magic_token::cpi::accounts::MintMagicToPlayer {
                    player: ctx.accounts.owner.to_account_info(),
                    game_config: ctx.accounts.game_config.to_account_info(),
                    caller_authority: ctx.accounts.marketplace_authority.to_account_info(),
                    program_authority: ctx.accounts.magic_token_authority.to_account_info(),
                    magic_token_mint: ctx.accounts.magic_token_mint.to_account_info(),
                    player_magic_token_account: ctx
                        .accounts
                        .player_magic_token_account
                        .to_account_info(),
                    token_program: ctx.accounts.magic_token_token_program.to_account_info(),
                },
                &[signer_seeds],
            ),
            ctx.accounts.game_config.item_prices[item_type.as_index()],
        )
    }
}

/// Empty bootstrap context used to confirm the program is deployed and configured.
#[derive(Accounts)]
pub struct Initialize {}

/// Accounts required to redeem an item NFT for configured reward tokens.
#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct RedeemItemForMagic<'info> {
    /// Current NFT owner authorizing the burn and paying ATA rent if needed.
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump = game_config.bump,
        constraint = usize::from(item_type) < ITEM_COUNT @ GameErrorCode::InvalidItemTypeIndex
    )]
    /// Shared pricing config used to validate the item type and reward amount.
    pub game_config: Account<'info, GameConfig>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: this PDA signs the CPI into magic_token.
    /// Marketplace PDA that signs CPIs into `item_nft` and `magic_token`.
    pub marketplace_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        seeds::program = MAGIC_TOKEN_PROGRAM_ID,
        bump
    )]
    /// CHECK: magic_token validates this PDA again before minting.
    /// Magic-token PDA that ultimately signs the reward mint CPI.
    pub magic_token_authority: UncheckedAccount<'info>,
    /// NFT mint being redeemed.
    #[account(mut)]
    pub mint: Account<'info, SplMint>,
    #[account(
        mut,
        seeds = [shared::ITEM_METADATA_SEED, mint.key().as_ref()],
        seeds::program = ITEM_NFT_PROGRAM_ID,
        bump = item_metadata.bump
    )]
    /// On-chain metadata account tracking the NFT owner and item type.
    pub item_metadata: Account<'info, ItemMetadata>,
    #[account(mut)]
    /// CHECK: item_nft validates the Metaplex metadata PDA.
    /// Metaplex metadata PDA for the NFT being burned.
    pub metadata: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: item_nft validates the Metaplex master edition PDA.
    /// Metaplex master edition PDA for the NFT being burned.
    pub master_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = owner,
        token::token_program = item_token_program
    )]
    /// Owner ATA currently holding the NFT.
    pub owner_item_token_account: Account<'info, TokenAccount>,
    /// Reward mint configured in `GameConfig`.
    #[account(mut)]
    pub magic_token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    /// CHECK: marketplace validates the expected ATA and creates it idempotently before minting.
    /// Owner ATA that receives the reward payout.
    pub player_magic_token_account: UncheckedAccount<'info>,
    #[account(address = TOKEN_METADATA_PROGRAM_ID)]
    /// CHECK: constrained to the canonical Metaplex Token Metadata program.
    /// Canonical Metaplex Token Metadata program.
    pub token_metadata_program: UncheckedAccount<'info>,
    /// CPI interface for burning the NFT.
    pub item_nft_program: Program<'info, ItemNft>,
    /// CPI interface for minting reward tokens.
    pub magic_token_program: Program<'info, MagicToken>,
    /// Associated token program used to create the reward ATA.
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// SPL Token program used by the NFT mint.
    pub item_token_program: Program<'info, Token>,
    /// Token program interface used by the reward mint.
    pub magic_token_token_program: Interface<'info, TokenInterface>,
    /// System program used when creating the reward ATA.
    pub system_program: Program<'info, System>,
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: constrained to the instructions sysvar address.
    /// Instructions sysvar required by Metaplex CPIs.
    pub sysvar_instructions: UncheckedAccount<'info>,
}
