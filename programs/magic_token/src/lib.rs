use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, MintTo, Token2022};

declare_id!("BQAqENU5HMGNF8Xunzbb859GCTz8v8Tuknqieqqk6ide");

#[program]
pub mod magic_token {
    use super::*;

    /// Ініціалізує MagicToken mint — викликається один раз адміном
    pub fn initialize_mint(_ctx: Context<InitializeMint>) -> Result<()> {
        msg!("MagicToken mint initialized");
        Ok(())
    }

    /// Мінтить MagicToken на акаунт гравця.
    /// Може викликатись ТІЛЬКИ через CPI з marketplace програми.
    pub fn mint_magic_token(ctx: Context<MintMagicToken>, amount: u64) -> Result<()> {
        let bump = ctx.accounts.mint_authority.bump;
        let seeds: &[&[u8]] = &[b"mint_authority", &[bump]];
        let signer_seeds = &[seeds];

        token_2022::mint_to(
            CpiContext::new_with_signer(
                anchor_spl::token_2022::ID,
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.player_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        msg!("Minted {} MagicTokens", amount);
        Ok(())
    }
}

/// PDA що є authority для мінту — тільки ця програма може мінтити
#[account]
pub struct MintAuthority {
    pub bump: u8,
}

impl MintAuthority {
    pub const LEN: usize = 8 + 1;
}

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(
        init,
        payer = admin,
        space = MintAuthority::LEN,
        seeds = [b"mint_authority"],
        bump
    )]
    pub mint_authority: Account<'info, MintAuthority>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintMagicToken<'info> {
    #[account(
        mut,
        seeds = [b"mint_authority"],
        bump = mint_authority.bump
    )]
    pub mint_authority: Account<'info, MintAuthority>,

    /// CHECK: це mint акаунт для MagicToken
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: токен акаунт гравця
    #[account(mut)]
    pub player_token_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
}