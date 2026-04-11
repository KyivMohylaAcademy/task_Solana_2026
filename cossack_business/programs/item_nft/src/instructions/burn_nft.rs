use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount},
};
use crate::{
    constants::AUTHORIZED_MARKETPLACE_PROGRAM,
    errors::ItemNftError,
    state::ItemMetadata,
};

#[derive(Accounts)]
pub struct BurnNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: CPI authority PDA from the calling program.
    /// Verified in handler against AUTHORIZED_MARKETPLACE_PROGRAM.
    pub cpi_auth: AccountInfo<'info>,

    /// The current holder of the NFT (must sign to authorize burn of their token).
    #[account(mut)]
    pub holder: Signer<'info>,

    /// The NFT mint.
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,

    /// Holder's ATA for the NFT. Thawed, burned, then closed.
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = holder,
    )]
    pub holder_nft_ata: Account<'info, TokenAccount>,

    /// ItemMetadata PDA — closed after burn, lamports returned to payer.
    #[account(
        mut,
        seeds = [b"item_metadata", nft_mint.key().as_ref()],
        bump = item_metadata.bump,
        close = payer,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// CHECK: PDA with freeze/thaw authority over NFT mints.
    #[account(seeds = [b"nft_authority"], bump)]
    pub nft_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Burn an NFT. Called via CPI from the marketplace program only.
/// Thaws the frozen ATA, burns the 1 token, then closes the empty ATA.
/// The ItemMetadata PDA is also closed.
pub fn handler(ctx: Context<BurnNft>) -> Result<()> {
    // Verify the caller is the authorized marketplace program.
    let expected = Pubkey::find_program_address(&[b"cpi_auth"], &AUTHORIZED_MARKETPLACE_PROGRAM).0;
    require_keys_eq!(
        ctx.accounts.cpi_auth.key(),
        expected,
        ItemNftError::Unauthorized
    );

    // Burn the 1 token.
    // NOTE: The ATA is not frozen — Metaplex revokes the freeze authority when creating the
    // master edition, so freeze/thaw is not used. Burn protection is enforced by the cpi_auth guard above.
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.nft_mint.to_account_info(),
                from: ctx.accounts.holder_nft_ata.to_account_info(),
                authority: ctx.accounts.holder.to_account_info(),
            },
        ),
        1,
    )?;

    // Close the (now empty) ATA.
    token::close_account(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        token::CloseAccount {
            account: ctx.accounts.holder_nft_ata.to_account_info(),
            destination: ctx.accounts.payer.to_account_info(),
            authority: ctx.accounts.holder.to_account_info(),
        },
    ))?;

    Ok(())
}
