use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
    associated_token::AssociatedToken,
};
use crate::{
    constants::{AUTHORIZED_CRAFTING_PROGRAM, AUTHORIZED_SEARCH_PROGRAM},
    errors::ResourceManagerError,
    state::GameConfig,
};

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct MintResource<'info> {
    /// The authorized caller's cpi_auth PDA (seeds: ["cpi_auth"] from the caller's program ID).
    /// CHECK: Verified against AUTHORIZED_SEARCH_PROGRAM or AUTHORIZED_CRAFTING_PROGRAM constants.
    pub cpi_auth: AccountInfo<'info>,

    /// GameConfig PDA — used to verify the mint address matches the resource_id.
    #[account(seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,

    /// The resource mint matching resource_id in GameConfig.
    #[account(
        mut,
        constraint = mint.key() == game_config.resource_mints[resource_id as usize] @ ResourceManagerError::InvalidMint,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    /// Recipient's ATA; initialized if it doesn't exist yet.
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
        associated_token::token_program = token_program,
    )]
    pub recipient_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The recipient wallet.
    /// CHECK: Only used as the ATA owner; no on-chain data needed.
    pub recipient: AccountInfo<'info>,

    /// PDA that holds mint authority over resource mints.
    /// CHECK: Seeds verified; used only as a signer in the mint CPI.
    #[account(seeds = [b"resource_mint_auth"], bump)]
    pub resource_mint_auth: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Mint `amount` of a resource token to `recipient`. Callable only by the search or crafting program.
pub fn handler(ctx: Context<MintResource>, resource_id: u8, amount: u64) -> Result<()> {
    require!(resource_id < 6, ResourceManagerError::InvalidResourceId);

    // Verify the caller is an authorized program.
    let expected_search = Pubkey::find_program_address(&[b"cpi_auth"], &AUTHORIZED_SEARCH_PROGRAM).0;
    let expected_crafting = Pubkey::find_program_address(&[b"cpi_auth"], &AUTHORIZED_CRAFTING_PROGRAM).0;
    let caller = ctx.accounts.cpi_auth.key();
    require!(
        caller == expected_search || caller == expected_crafting,
        ResourceManagerError::Unauthorized
    );

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
