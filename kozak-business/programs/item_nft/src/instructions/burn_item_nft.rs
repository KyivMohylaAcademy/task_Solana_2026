use anchor_lang::prelude::*;
use anchor_spl::{
    metadata::{burn_nft, mpl_token_metadata, BurnNft, Metadata},
    token::{Token, TokenAccount},
};

use crate::{
    constants::{ITEM_CONFIG_SEED, MARKETPLACE_AUTHORITY_SEED},
    error::ItemNftError,
    state::ItemConfig,
};

/// Accounts for [`handler`]. Only the registered `marketplace` program can
/// satisfy `marketplace_authority`, mirroring the `search_authority` pattern.
#[derive(Accounts)]
pub struct BurnItemNft<'info> {
    #[account(
        seeds = [ITEM_CONFIG_SEED],
        bump = item_config.bump,
    )]
    pub item_config: Account<'info, ItemConfig>,

    /// PDA owned by the registered marketplace program — proves the call
    /// originated there. Nobody else can forge a signature for this PDA.
    ///
    /// CHECK: Constraint enforced by Anchor via seeds + seeds::program.
    #[account(
        seeds = [MARKETPLACE_AUTHORITY_SEED],
        bump,
        seeds::program = item_config.marketplace_program,
    )]
    pub marketplace_authority: Signer<'info>,

    /// The NFT holder / token account owner. Must sign so the holder consents
    /// to the burn (marketplace can't unilaterally burn items).
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The NFT mint.
    ///
    /// CHECK: Metaplex validates this matches `metadata.mint`.
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    /// The holder's token account for this NFT.
    #[account(
        mut,
        token::mint = mint,
        token::authority = owner,
        token::token_program = token_program,
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// Metaplex metadata PDA for this mint.
    ///
    /// CHECK: Validated by the Metaplex Token Metadata program.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// Metaplex master edition PDA for this mint.
    ///
    /// CHECK: Validated by the Metaplex Token Metadata program.
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
}

/// Burn an NFT (token + metadata + edition accounts).
///
/// The double-gating pattern:
///   1. `marketplace_authority` must be a PDA derived under the registered
///      `marketplace` program — only that program can CPI here.
///   2. `owner` must sign — the holder can't have their NFT burnt without
///      their signature propagating from the outer transaction.
pub fn handler(ctx: Context<BurnItemNft>) -> Result<()> {
    require!(
        ctx.accounts.item_config.marketplace_program != Pubkey::default(),
        ItemNftError::MarketplaceProgramNotRegistered
    );

    burn_nft(
        CpiContext::new(
            mpl_token_metadata::ID,
            BurnNft {
                metadata: ctx.accounts.metadata.to_account_info(),
                owner: ctx.accounts.owner.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                token: ctx.accounts.token_account.to_account_info(),
                edition: ctx.accounts.master_edition.to_account_info(),
                spl_token: ctx.accounts.token_program.to_account_info(),
            },
        ),
        None, // no collection metadata
    )?;

    msg!(
        "Burned NFT {} from owner {}",
        ctx.accounts.mint.key(),
        ctx.accounts.owner.key()
    );
    Ok(())
}
