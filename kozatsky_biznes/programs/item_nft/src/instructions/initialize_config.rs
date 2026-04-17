use anchor_lang::prelude::*;
use crate::state::*;

/// Ініціалізує конфігурацію ItemNft та PDA-авторитет мінтингу.
///
/// # Arguments
/// * `crafting_authority`   — адреса PDA програми crafting (seeds=["crafting_authority"])
/// * `marketplace_authority` — адреса PDA програми marketplace (seeds=["marketplace_authority"])
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    /// Конфігурація ItemNft (PDA)
    #[account(
        init,
        payer = admin,
        space = ItemNftConfig::SPACE,
        seeds = [b"item_nft_config"],
        bump,
    )]
    pub item_nft_config: Account<'info, ItemNftConfig>,

    /// PDA-авторитет для підписання mint/burn операцій NFT
    #[account(
        init,
        payer = admin,
        space = NftMintAuthority::SPACE,
        seeds = [b"nft_mint_authority"],
        bump,
    )]
    pub nft_mint_authority: Account<'info, NftMintAuthority>,

    /// Адміністратор (платник)
    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeConfig>,
    crafting_authority: Pubkey,
    marketplace_authority: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.item_nft_config;
    config.admin = ctx.accounts.admin.key();
    config.crafting_authority = crafting_authority;
    config.marketplace_authority = marketplace_authority;
    config.bump = ctx.bumps.item_nft_config;

    ctx.accounts.nft_mint_authority.bump = ctx.bumps.nft_mint_authority;

    msg!(
        "ItemNft конфігурацію ініціалізовано. Crafting: {}, Marketplace: {}",
        crafting_authority,
        marketplace_authority
    );
    Ok(())
}
