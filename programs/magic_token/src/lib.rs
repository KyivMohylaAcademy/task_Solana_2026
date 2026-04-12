/// magic_token — minting authority for the MagicToken (SPL Token-2022).
///
/// MagicToken is the game's reward currency.  It can ONLY be minted through
/// this program's `mint_to_player` instruction.  That instruction requires the
/// `marketplace_authority` PDA (owned by the marketplace program) as a signer,
/// ensuring that only the Marketplace can trigger minting.
///
/// Direct minting via the Token-2022 Program is impossible because the mint
/// authority is a PDA of *this* program (`seeds = [b"magic_mint_authority"]`).
use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{self, MintTo, Token2022},
    token_interface::{Mint, TokenAccount},
};

declare_id!("MagcTok1111111111111111111111111111111111111");

// ─── Program ──────────────────────────────────────────────────────────────────

#[program]
pub mod magic_token {
    use super::*;

    /// Initialises the [`MagicTokenConfig`] account.
    ///
    /// `marketplace_authority` is the PDA of the marketplace program
    /// (`seeds = [b"marketplace_authority"]`, bump derived from marketplace program).
    pub fn initialize(
        ctx: Context<Initialize>,
        marketplace_authority: Pubkey,
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.magic_token_config;
        cfg.admin                 = ctx.accounts.admin.key();
        cfg.mint                  = ctx.accounts.magic_mint.key();
        cfg.marketplace_authority = marketplace_authority;
        cfg.total_minted          = 0;
        cfg.bump                  = ctx.bumps.magic_token_config;
        Ok(())
    }

    /// Mints `amount` MagicTokens to `player_token_account`.
    ///
    /// The `marketplace_authority` must sign — only the marketplace program
    /// can provide this signature via CPI with its own PDA signer seeds.
    pub fn mint_to_player(
        ctx: Context<MintToPlayer>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, MagicTokenError::ZeroAmount);

        let cfg = &ctx.accounts.magic_token_config;

        // ── Access control: only marketplace_authority may call this ──────────
        require_keys_eq!(
            ctx.accounts.marketplace_authority.key(),
            cfg.marketplace_authority,
            MagicTokenError::Unauthorised
        );

        // ── Sign with magic_mint_authority PDA ────────────────────────────────
        let seeds: &[&[u8]] = &[b"magic_mint_authority", &[ctx.bumps.magic_mint_authority]];
        let signer_seeds = &[seeds];

        token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint:      ctx.accounts.magic_mint.to_account_info(),
                    to:        ctx.accounts.player_token_account.to_account_info(),
                    authority: ctx.accounts.magic_mint_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        ctx.accounts.magic_token_config.total_minted = ctx
            .accounts.magic_token_config.total_minted
            .checked_add(amount)
            .ok_or(MagicTokenError::Overflow)?;

        emit!(MagicTokenMinted {
            player: ctx.accounts.player_token_account.owner,
            amount,
        });
        Ok(())
    }
}

// ─── Accounts ─────────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer  = admin,
        space  = MagicTokenConfig::LEN,
        seeds  = [b"magic_token_config"],
        bump,
    )]
    pub magic_token_config: Account<'info, MagicTokenConfig>,

    /// The MagicToken SPL Token-2022 mint.
    /// Must have been created with `magic_mint_authority` PDA as mint_authority.
    pub magic_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintToPlayer<'info> {
    #[account(seeds = [b"magic_token_config"], bump = magic_token_config.bump)]
    pub magic_token_config: Account<'info, MagicTokenConfig>,

    /// This program's mint authority PDA.
    /// CHECK: PDA derived with known seeds — used only for signing.
    #[account(seeds = [b"magic_mint_authority"], bump)]
    pub magic_mint_authority: UncheckedAccount<'info>,

    /// Must equal `magic_token_config.marketplace_authority`.
    pub marketplace_authority: Signer<'info>,

    #[account(
        mut,
        address = magic_token_config.mint,
    )]
    pub magic_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
}

// ─── State ────────────────────────────────────────────────────────────────────

/// Configuration for the MagicToken mint.
#[account]
pub struct MagicTokenConfig {
    pub admin: Pubkey,
    /// The SPL Token-2022 mint address.
    pub mint: Pubkey,
    /// PDA of the marketplace program — the only authorised minter.
    pub marketplace_authority: Pubkey,
    /// Lifetime total minted (for analytics / anti-inflation checks).
    pub total_minted: u64,
    pub bump: u8,
}

impl MagicTokenConfig {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 1;
}

// ─── Events ───────────────────────────────────────────────────────────────────

#[event]
pub struct MagicTokenMinted {
    pub player: Pubkey,
    pub amount: u64,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[error_code]
pub enum MagicTokenError {
    #[msg("Only the marketplace authority may mint MagicTokens")]
    Unauthorised,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Arithmetic overflow")]
    Overflow,
}
