use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
    token_2022::Token2022,
    token_interface::Mint as InterfaceMint,
};
use item_nft::{cpi::accounts::BurnNft, state::ItemMetadata};
use magic_token::cpi::accounts::MintMagicToken;
use resource_manager::state::GameConfig;
use crate::errors::MarketplaceError;

#[derive(Accounts)]
pub struct SellItem<'info> {
    /// The seller — must sign, must hold the NFT.
    #[account(mut)]
    pub seller: Signer<'info>,

    /// GameConfig — provides item_prices.
    #[account(seeds = [b"game_config"], bump = game_config.bump, seeds::program = resource_manager::ID)]
    pub game_config: Box<Account<'info, GameConfig>>,

    /// The NFT mint (classic SPL Token).
    #[account(mut)]
    pub nft_mint: Box<Account<'info, Mint>>,

    /// Seller's ATA for the NFT. Must have balance ≥ 1.
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = seller,
    )]
    pub seller_nft_ata: Box<Account<'info, TokenAccount>>,

    /// ItemMetadata PDA — provides item_type for price lookup; closed by burn_nft CPI.
    #[account(
        mut,
        seeds = [b"item_metadata", nft_mint.key().as_ref()],
        bump = item_metadata.bump,
        seeds::program = item_nft::ID,
    )]
    pub item_metadata: Box<Account<'info, ItemMetadata>>,

    /// CHECK: nft_authority PDA from item_nft — holds freeze/thaw authority.
    #[account(seeds = [b"nft_authority"], bump, seeds::program = item_nft::ID)]
    pub nft_authority: AccountInfo<'info>,

    /// CHECK: This program's cpi_auth PDA — passed to item_nft and magic_token as caller identity.
    #[account(seeds = [b"cpi_auth"], bump)]
    pub cpi_auth: AccountInfo<'info>,

    /// MagicToken mint (Token-2022).
    #[account(
        mut,
        constraint = magic_token_mint.key() == game_config.magic_token_mint,
    )]
    pub magic_token_mint: Box<InterfaceAccount<'info, InterfaceMint>>,

    /// CHECK: Seller's MagicToken ATA; created by magic_token CPI if needed (init_if_needed there).
    #[account(mut)]
    pub seller_magic_ata: AccountInfo<'info>,

    /// CHECK: magic_mint_auth PDA from magic_token program.
    #[account(seeds = [b"magic_mint_auth"], bump, seeds::program = magic_token::ID)]
    pub magic_mint_auth: AccountInfo<'info>,

    /// CHECK: item_nft program.
    #[account(address = item_nft::ID)]
    pub item_nft_program: AccountInfo<'info>,

    /// CHECK: magic_token program.
    #[account(address = magic_token::ID)]
    pub magic_token_program: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Sell an NFT to the game. Burns the NFT, mints MagicToken to seller at GameConfig price.
pub fn handler(ctx: Context<SellItem>) -> Result<()> {
    // Verify the seller holds the NFT.
    require!(
        ctx.accounts.seller_nft_ata.amount >= 1,
        MarketplaceError::NotNftHolder
    );

    let item_type = ctx.accounts.item_metadata.item_type;
    require!(item_type < 4, MarketplaceError::InvalidItemType);

    let price = ctx.accounts.game_config.item_prices[item_type as usize];

    let cpi_bump = ctx.bumps.cpi_auth;
    let signer_seeds: &[&[&[u8]]] = &[&[b"cpi_auth", &[cpi_bump]]];

    // CPI → item_nft::burn_nft
    let burn_accounts = BurnNft {
        payer: ctx.accounts.seller.to_account_info(),
        cpi_auth: ctx.accounts.cpi_auth.to_account_info(),
        holder: ctx.accounts.seller.to_account_info(),
        nft_mint: ctx.accounts.nft_mint.to_account_info(),
        holder_nft_ata: ctx.accounts.seller_nft_ata.to_account_info(),
        item_metadata: ctx.accounts.item_metadata.to_account_info(),
        nft_authority: ctx.accounts.nft_authority.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    item_nft::cpi::burn_nft(CpiContext::new_with_signer(
        ctx.accounts.item_nft_program.to_account_info(),
        burn_accounts,
        signer_seeds,
    ))?;

    // CPI → magic_token::mint_magic_token
    let mint_accounts = MintMagicToken {
        cpi_auth: ctx.accounts.cpi_auth.to_account_info(),
        mint: ctx.accounts.magic_token_mint.to_account_info(),
        recipient_ata: ctx.accounts.seller_magic_ata.to_account_info(),
        recipient: ctx.accounts.seller.to_account_info(),
        magic_mint_auth: ctx.accounts.magic_mint_auth.to_account_info(),
        payer: ctx.accounts.seller.to_account_info(),
        token_program: ctx.accounts.token_2022_program.to_account_info(),
        associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    magic_token::cpi::mint_magic_token(
        CpiContext::new_with_signer(
            ctx.accounts.magic_token_program.to_account_info(),
            mint_accounts,
            signer_seeds,
        ),
        price,
    )?;

    Ok(())
}
