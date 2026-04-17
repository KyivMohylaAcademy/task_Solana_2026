use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Burn, Token2022};

use crate::state::*;
use crate::error::ItemNftError;

/// Спалює NFT предмет з акаунту гравця та закриває ItemMetadata PDA.
///
/// Безпека: викликається виключно через CPI з програми marketplace.
/// Верифікація: `caller_authority` має бути signer та відповідати
/// `item_nft_config.marketplace_authority`.
#[derive(Accounts)]
pub struct BurnNftItem<'info> {
    /// Конфігурація ItemNft
    #[account(
        seeds = [b"item_nft_config"],
        bump = item_nft_config.bump,
    )]
    pub item_nft_config: Account<'info, ItemNftConfig>,

    /// Авторизований виклик (marketplace authority PDA)
    /// CHECK: Перевіряється в коді: має бути signer та відповідати marketplace_authority
    #[account(
        signer,
        constraint = caller_authority.key() == item_nft_config.marketplace_authority
            @ ItemNftError::UnauthorizedBurnCaller
    )]
    pub caller_authority: AccountInfo<'info>,

    /// Гравець-власник NFT (підписує burn)
    #[account(mut)]
    pub player: Signer<'info>,

    /// NFT мінт
    /// CHECK: Перевіряється через item_metadata.mint
    #[account(
        mut,
        constraint = nft_mint.key() == item_metadata.mint @ ItemNftError::MintMismatch
    )]
    pub nft_mint: AccountInfo<'info>,

    /// Метадані предмету (буде закрито після спалення)
    #[account(
        mut,
        seeds = [b"item_metadata", nft_mint.key().as_ref()],
        bump = item_metadata.bump,
        close = player,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// Токен-акаунт гравця з NFT
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = player,
        associated_token::token_program = token_2022_program,
    )]
    pub player_nft_token_account: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    pub token_2022_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<BurnNftItem>) -> Result<()> {
    let item_type = ctx.accounts.item_metadata.item_type;

    // Спалюємо NFT (1 токен) з акаунту гравця
    token_2022::burn(
        CpiContext::new(
            anchor_spl::token_2022::ID,
            Burn {
                mint: ctx.accounts.nft_mint.to_account_info(),
                from: ctx.accounts.player_nft_token_account.to_account_info(),
                authority: ctx.accounts.player.to_account_info(),
            },
        ),
        1,
    )?;

    msg!(
        "NFT предмет типу {} ({}) спалено у гравця {}",
        item_type,
        ctx.accounts.nft_mint.key(),
        ctx.accounts.player.key()
    );
    Ok(())
}
