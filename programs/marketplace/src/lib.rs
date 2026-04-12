/// marketplace — allows players to sell item NFTs in exchange for MagicToken.
///
/// Flow
/// ----
/// 1. `list_item`  — Player places their item NFT into the marketplace escrow.
///                   A [`Listing`] PDA is created.
/// 2. `cancel_listing` — Seller reclaims the item before a sale.
/// 3. `sell_item`  — (no buyer needed) The player directly redeems their
///                   listed item for newly minted MagicTokens.
///                   The item NFT is burned via CPI to `item_nft`.
///                   MagicTokens are minted via CPI to `magic_token`.
///
/// Design note
/// -----------
/// Based on the spec ("NFT спалюється, продавець отримує MagicToken") the
/// marketplace acts as a game "shop": crafted items are sold *to the game* for
/// MagicToken rewards.  There is no buyer; the item is burned and MagicToken is
/// freshly minted as the reward.  Prices per item type come from
/// resource_manager's `GameConfig.item_prices`.
use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{
        self, Token2022,
        Transfer, transfer,
    },
    token_interface::{Mint, TokenAccount},
};
use magic_token::{
    self,
    cpi::{accounts::MintToPlayer, mint_to_player},
    program::MagicToken,
};
use item_nft::{
    self,
    cpi::{accounts::BurnItem, burn_item},
    program::ItemNft,
    ItemMetadata,
};

declare_id!("MktPlace111111111111111111111111111111111111");

// ─── Program ──────────────────────────────────────────────────────────────────

#[program]
pub mod marketplace {
    use super::*;

    /// Initialises the [`MarketplaceConfig`] PDA.
    pub fn initialize(
        ctx: Context<Initialize>,
        magic_token_prices: [u64; 4],
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.marketplace_config;
        cfg.admin              = ctx.accounts.admin.key();
        cfg.magic_token_prices = magic_token_prices;
        cfg.total_sales        = 0;
        cfg.bump               = ctx.bumps.marketplace_config;
        Ok(())
    }

    /// Lists an item NFT for sale.
    ///
    /// Transfers the NFT from `seller_token_account` to the marketplace's
    /// escrow token account (a PDA-owned ATA).
    ///
    /// Note: Because item token accounts are frozen by `item_nft`, they must
    /// first be thawed by the seller (or this program) before transfer.
    /// In practice the player initiates a CPI to `item_nft` to thaw then
    /// transfers here; for simplicity we assume the account is already thawed
    /// by the time this instruction runs.
    pub fn list_item(ctx: Context<ListItem>) -> Result<()> {
        // Verify item_metadata belongs to this mint
        require_keys_eq!(
            ctx.accounts.item_metadata.mint,
            ctx.accounts.item_mint.key(),
            MarketplaceError::MintMismatch
        );
        require_keys_eq!(
            ctx.accounts.item_metadata.owner,
            ctx.accounts.seller.key(),
            MarketplaceError::NotOwner
        );

        let item_type = ctx.accounts.item_metadata.item_type;

        // Transfer NFT to escrow
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from:      ctx.accounts.seller_token_account.to_account_info(),
                    to:        ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            1,
        )?;

        // Create listing
        let listing     = &mut ctx.accounts.listing;
        listing.seller  = ctx.accounts.seller.key();
        listing.item_mint = ctx.accounts.item_mint.key();
        listing.item_type = item_type;
        listing.is_active = true;
        listing.bump      = ctx.bumps.listing;

        emit!(ItemListed {
            seller:    ctx.accounts.seller.key(),
            item_mint: ctx.accounts.item_mint.key(),
            item_type,
        });
        Ok(())
    }

    /// Cancels a listing, returning the item to the seller.
    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        require!(ctx.accounts.listing.is_active, MarketplaceError::ListingInactive);
        require_keys_eq!(
            ctx.accounts.listing.seller,
            ctx.accounts.seller.key(),
            MarketplaceError::NotOwner
        );

        let seeds: &[&[u8]] = &[b"escrow_authority", &[ctx.bumps.escrow_authority]];
        let signer = &[seeds];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from:      ctx.accounts.escrow_token_account.to_account_info(),
                    to:        ctx.accounts.seller_token_account.to_account_info(),
                    authority: ctx.accounts.escrow_authority.to_account_info(),
                },
                signer,
            ),
            1,
        )?;

        ctx.accounts.listing.is_active = false;

        emit!(ListingCancelled {
            seller:    ctx.accounts.seller.key(),
            item_mint: ctx.accounts.listing.item_mint,
        });
        Ok(())
    }

    /// Sells (redeems) a listed item NFT for MagicTokens.
    ///
    /// Steps:
    ///   1. Transfer NFT from escrow back to seller (needed for burn).
    ///   2. Burn the item NFT via CPI to `item_nft`.
    ///   3. Mint MagicTokens to seller via CPI to `magic_token`.
    ///   4. Close listing.
    pub fn sell_item(ctx: Context<SellItem>) -> Result<()> {
        require!(ctx.accounts.listing.is_active, MarketplaceError::ListingInactive);
        require_keys_eq!(
            ctx.accounts.listing.seller,
            ctx.accounts.seller.key(),
            MarketplaceError::NotOwner
        );
        require_keys_eq!(
            ctx.accounts.listing.item_mint,
            ctx.accounts.item_mint.key(),
            MarketplaceError::MintMismatch
        );

        let item_type = ctx.accounts.listing.item_type as usize;
        let price     = ctx.accounts.marketplace_config.magic_token_prices[item_type];

        let escrow_auth_bump = ctx.bumps.escrow_authority;
        let mp_auth_bump     = ctx.bumps.marketplace_authority;

        let escrow_seeds: &[&[u8]] = &[b"escrow_authority", &[escrow_auth_bump]];
        let mp_seeds:     &[&[u8]] = &[b"marketplace_authority", &[mp_auth_bump]];

        // ── 1. Transfer NFT from escrow back to seller ────────────────────────
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from:      ctx.accounts.escrow_token_account.to_account_info(),
                    to:        ctx.accounts.seller_token_account.to_account_info(),
                    authority: ctx.accounts.escrow_authority.to_account_info(),
                },
                &[escrow_seeds],
            ),
            1,
        )?;

        // ── 2. Burn item NFT ──────────────────────────────────────────────────
        burn_item(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                BurnItem {
                    item_nft_config:        ctx.accounts.item_nft_config.to_account_info(),
                    item_nft_authority:     ctx.accounts.item_nft_authority.to_account_info(),
                    marketplace_authority:  ctx.accounts.marketplace_authority.to_account_info(),
                    player:                 ctx.accounts.seller.to_account_info(),
                    item_mint:              ctx.accounts.item_mint.to_account_info(),
                    player_token_account:   ctx.accounts.seller_token_account.to_account_info(),
                    item_metadata:          ctx.accounts.item_metadata.to_account_info(),
                    token_program:          ctx.accounts.token_program.to_account_info(),
                },
                &[mp_seeds],
            ),
        )?;

        // ── 3. Mint MagicTokens to seller ─────────────────────────────────────
        mint_to_player(
            CpiContext::new_with_signer(
                ctx.accounts.magic_token_program.to_account_info(),
                MintToPlayer {
                    magic_token_config:     ctx.accounts.magic_token_config.to_account_info(),
                    magic_mint_authority:   ctx.accounts.magic_mint_authority.to_account_info(),
                    marketplace_authority:  ctx.accounts.marketplace_authority.to_account_info(),
                    magic_mint:             ctx.accounts.magic_mint.to_account_info(),
                    player_token_account:   ctx.accounts.seller_magic_token_account.to_account_info(),
                    token_program:          ctx.accounts.token_program.to_account_info(),
                },
                &[mp_seeds],
            ),
            price,
        )?;

        // ── 4. Close listing ──────────────────────────────────────────────────
        ctx.accounts.listing.is_active = false;
        ctx.accounts.marketplace_config.total_sales = ctx
            .accounts.marketplace_config.total_sales
            .saturating_add(1);

        emit!(ItemSold {
            seller:         ctx.accounts.seller.key(),
            item_mint:      ctx.accounts.item_mint.key(),
            magic_token_reward: price,
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
        space  = MarketplaceConfig::LEN,
        seeds  = [b"marketplace_config"],
        bump,
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ListItem<'info> {
    #[account(seeds = [b"marketplace_config"], bump = marketplace_config.bump)]
    pub marketplace_config: Account<'info, MarketplaceConfig>,

    #[account(
        init,
        payer  = seller,
        space  = Listing::LEN,
        seeds  = [b"listing", item_mint.key().as_ref()],
        bump,
    )]
    pub listing: Account<'info, Listing>,

    /// CHECK: derived via seeds
    #[account(seeds = [b"escrow_authority"], bump)]
    pub escrow_authority: UncheckedAccount<'info>,

    pub seller: Signer<'info>,

    pub item_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub seller_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub escrow_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Item metadata — used to verify ownership and get item_type.
    pub item_metadata: Account<'info, ItemMetadata>,

    pub token_program:  Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    pub seller: Signer<'info>,

    #[account(
        mut,
        seeds = [b"listing", listing.item_mint.as_ref()],
        bump  = listing.bump,
    )]
    pub listing: Account<'info, Listing>,

    /// CHECK: derived via seeds
    #[account(seeds = [b"escrow_authority"], bump)]
    pub escrow_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub seller_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub escrow_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct SellItem<'info> {
    #[account(mut, seeds = [b"marketplace_config"], bump = marketplace_config.bump)]
    pub marketplace_config: Account<'info, MarketplaceConfig>,

    #[account(
        mut,
        seeds = [b"listing", item_mint.key().as_ref()],
        bump  = listing.bump,
    )]
    pub listing: Account<'info, Listing>,

    /// CHECK: derived via seeds — signs escrow transfer
    #[account(seeds = [b"escrow_authority"], bump)]
    pub escrow_authority: UncheckedAccount<'info>,

    /// CHECK: derived via seeds — signs item_nft and magic_token CPIs
    #[account(seeds = [b"marketplace_authority"], bump)]
    pub marketplace_authority: UncheckedAccount<'info>,

    pub seller: Signer<'info>,

    pub item_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub seller_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub escrow_token_account: InterfaceAccount<'info, TokenAccount>,

    /// item_nft program accounts
    /// CHECK: validated inside item_nft CPI
    pub item_nft_config: UncheckedAccount<'info>,
    /// CHECK: validated inside item_nft CPI
    pub item_nft_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// magic_token program accounts
    /// CHECK: validated inside magic_token CPI
    pub magic_token_config: UncheckedAccount<'info>,
    /// CHECK: validated inside magic_token CPI
    pub magic_mint_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub magic_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub seller_magic_token_account: InterfaceAccount<'info, TokenAccount>,

    pub item_nft_program:    Program<'info, ItemNft>,
    pub magic_token_program: Program<'info, MagicToken>,
    pub token_program:       Program<'info, Token2022>,
}

// ─── State ────────────────────────────────────────────────────────────────────

#[account]
pub struct MarketplaceConfig {
    pub admin: Pubkey,
    /// MagicToken reward per item type: [saber, staff, armor, bracelet].
    pub magic_token_prices: [u64; 4],
    pub total_sales: u64,
    pub bump: u8,
}

impl MarketplaceConfig {
    pub const LEN: usize = 8 + 32 + 8 * 4 + 8 + 1;
}

/// Represents a single listed item.
#[account]
pub struct Listing {
    pub seller:     Pubkey,
    pub item_mint:  Pubkey,
    pub item_type:  u8,
    pub is_active:  bool,
    pub bump:       u8,
}

impl Listing {
    pub const LEN: usize = 8 + 32 + 32 + 1 + 1 + 1;
}

// ─── Events ───────────────────────────────────────────────────────────────────

#[event]
pub struct ItemListed {
    pub seller:    Pubkey,
    pub item_mint: Pubkey,
    pub item_type: u8,
}

#[event]
pub struct ListingCancelled {
    pub seller:    Pubkey,
    pub item_mint: Pubkey,
}

#[event]
pub struct ItemSold {
    pub seller:             Pubkey,
    pub item_mint:          Pubkey,
    pub magic_token_reward: u64,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[error_code]
pub enum MarketplaceError {
    #[msg("Listing is not active")]
    ListingInactive,
    #[msg("Caller is not the item owner")]
    NotOwner,
    #[msg("Mint does not match listing")]
    MintMismatch,
}
