use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
    associated_token::AssociatedToken,
};
use crate::{
    errors::ResourceManagerError,
    state::GameConfig,
};

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct AdminMintResource<'info> {
    /// Must be the GameConfig admin (deployer wallet).
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        has_one = admin @ ResourceManagerError::Unauthorized,
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(
        mut,
        constraint = mint.key() == game_config.resource_mints[resource_id as usize] @ ResourceManagerError::InvalidMint,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = recipient,
        associated_token::token_program = token_program,
    )]
    pub recipient_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Only used as ATA owner.
    pub recipient: AccountInfo<'info>,

    /// CHECK: Seeds verified; used only as a signer in the mint CPI.
    #[account(seeds = [b"resource_mint_auth"], bump)]
    pub resource_mint_auth: AccountInfo<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Admin-only resource minting. Used exclusively in test setup for crafting/marketplace tests.
/// Gated by GameConfig.admin — not a public minting path.
pub fn handler(ctx: Context<AdminMintResource>, resource_id: u8, amount: u64) -> Result<()> {
    require!(resource_id < 6, ResourceManagerError::InvalidResourceId);

    let bump = ctx.bumps.resource_mint_auth;
    let signer_seeds: &[&[&[u8]]] = &[&[b"resource_mint_auth", &[bump]]];

    anchor_spl::token_2022::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_2022::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.resource_mint_auth.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    Ok(())
}
