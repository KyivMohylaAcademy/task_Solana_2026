use anchor_lang::prelude::*;
use crate::state::*;

/// Ініціалізує PDA SearchAuthority (викликається один раз адміністратором).
///
/// Ця PDA реєструється в resource_manager GameConfig як `search_authority`,
/// щоб дозволити search програмі виконувати CPI до mint_resource.
#[derive(Accounts)]
pub struct InitializeSearchAuthority<'info> {
    /// SearchAuthority PDA
    #[account(
        init,
        payer = admin,
        space = SearchAuthority::SPACE,
        seeds = [b"search_authority"],
        bump,
    )]
    pub search_authority: Account<'info, SearchAuthority>,

    /// Адміністратор (платник)
    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeSearchAuthority>) -> Result<()> {
    ctx.accounts.search_authority.bump = ctx.bumps.search_authority;

    msg!(
        "SearchAuthority ініціалізовано: {}",
        ctx.accounts.search_authority.key()
    );
    Ok(())
}
