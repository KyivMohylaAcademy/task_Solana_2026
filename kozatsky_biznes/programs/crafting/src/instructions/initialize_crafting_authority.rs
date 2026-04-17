use anchor_lang::prelude::*;
use crate::state::*;

/// Ініціалізує PDA CraftingAuthority (викликається один раз адміністратором).
///
/// Ця PDA реєструється в resource_manager GameConfig як `crafting_authority`,
/// а також в item_nft як авторизований виклик для mint_nft_item.
#[derive(Accounts)]
pub struct InitializeCraftingAuthority<'info> {
    /// CraftingAuthority PDA
    #[account(
        init,
        payer = admin,
        space = CraftingAuthority::SPACE,
        seeds = [b"crafting_authority"],
        bump,
    )]
    pub crafting_authority: Account<'info, CraftingAuthority>,

    /// Адміністратор (платник)
    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeCraftingAuthority>) -> Result<()> {
    ctx.accounts.crafting_authority.bump = ctx.bumps.crafting_authority;

    msg!(
        "CraftingAuthority ініціалізовано: {}",
        ctx.accounts.crafting_authority.key()
    );
    Ok(())
}
