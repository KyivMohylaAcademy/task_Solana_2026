use anchor_lang::prelude::*;
use crate::state::*;

/// Ініціалізує PDA MarketplaceAuthority (викликається один раз адміністратором).
///
/// Ця PDA реєструється в item_nft як авторизований виклик для burn_nft_item,
/// а також в magic_token як авторизований виклик для mint_magic_token.
#[derive(Accounts)]
pub struct InitializeMarketplaceAuthority<'info> {
    /// MarketplaceAuthority PDA
    #[account(
        init,
        payer = admin,
        space = MarketplaceAuthority::SPACE,
        seeds = [b"marketplace_authority"],
        bump,
    )]
    pub marketplace_authority: Account<'info, MarketplaceAuthority>,

    /// Адміністратор (платник)
    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeMarketplaceAuthority>) -> Result<()> {
    ctx.accounts.marketplace_authority.bump = ctx.bumps.marketplace_authority;

    msg!(
        "MarketplaceAuthority ініціалізовано: {}",
        ctx.accounts.marketplace_authority.key()
    );
    Ok(())
}
