use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};
use crate::{
    constants::AUTHORIZED_MARKETPLACE_PROGRAM,
    errors::MagicTokenError,
};

#[derive(Accounts)]
pub struct MintMagicToken<'info> {
    /// The marketplace program's cpi_auth PDA.
    /// CHECK: Verified against AUTHORIZED_MARKETPLACE_PROGRAM constant.
    pub cpi_auth: AccountInfo<'info>,

    #[account(mut)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
        associated_token::token_program = token_program,
    )]
    pub recipient_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Only used as the ATA owner.
    pub recipient: AccountInfo<'info>,

    /// CHECK: Seeds verified; used only as a signer in the mint CPI.
    #[account(seeds = [b"magic_mint_auth"], bump)]
    pub magic_mint_auth: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Mint MagicToken to recipient. Callable only by the marketplace program.
pub fn handler(ctx: Context<MintMagicToken>, amount: u64) -> Result<()> {
    let expected = Pubkey::find_program_address(
        &[b"cpi_auth"],
        &AUTHORIZED_MARKETPLACE_PROGRAM,
    ).0;
    require_keys_eq!(
        ctx.accounts.cpi_auth.key(),
        expected,
        MagicTokenError::Unauthorized
    );

    let bump = ctx.bumps.magic_mint_auth;
    let signer_seeds: &[&[&[u8]]] = &[&[b"magic_mint_auth", &[bump]]];

    anchor_spl::token_2022::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_2022::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.magic_mint_auth.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    Ok(())
}
