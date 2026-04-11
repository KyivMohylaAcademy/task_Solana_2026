use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};
use crate::{
    constants::AUTHORIZED_CRAFTING_PROGRAM,
    errors::ResourceManagerError,
    state::GameConfig,
};

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct BurnResource<'info> {
    /// The authorized caller's cpi_auth PDA. Must be from the crafting program.
    /// CHECK: Verified against AUTHORIZED_CRAFTING_PROGRAM constant.
    pub cpi_auth: AccountInfo<'info>,

    #[account(seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,

    #[account(
        mut,
        constraint = mint.key() == game_config.resource_mints[resource_id as usize] @ ResourceManagerError::InvalidMint,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    /// The owner's ATA to burn from. Must already exist.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program,
    )]
    pub source_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The token holder (must sign — SPL Token enforces ATA ownership).
    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token2022>,
}

/// Burn `amount` of a resource token from `owner`'s ATA. Callable only by the crafting program.
pub fn handler(ctx: Context<BurnResource>, resource_id: u8, amount: u64) -> Result<()> {
    require!(resource_id < 6, ResourceManagerError::InvalidResourceId);

    let expected = Pubkey::find_program_address(&[b"cpi_auth"], &AUTHORIZED_CRAFTING_PROGRAM).0;
    require_keys_eq!(
        ctx.accounts.cpi_auth.key(),
        expected,
        ResourceManagerError::Unauthorized
    );

    anchor_spl::token_2022::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_2022::Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.source_ata.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        amount,
    )?;

    Ok(())
}
