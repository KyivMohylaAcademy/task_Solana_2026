use anchor_lang::prelude::*;
use crate::state::MagicTokenConfig;

/// Accounts for initializing the MagicToken mint (Token-2022 + MetadataPointer).
#[derive(Accounts)]
pub struct InitializeMagicToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + MagicTokenConfig::INIT_SPACE,
        seeds = [b"magic_config"],
        bump,
    )]
    pub config: Account<'info, MagicTokenConfig>,

    /// New mint keypair
    #[account(mut)]
    pub mint: Signer<'info>,

    /// CHECK: PDA used as mint authority for MagicToken
    #[account(seeds = [b"magic_mint_authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Token-2022 program
    #[account(address = spl_token_2022::ID)]
    pub token_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
