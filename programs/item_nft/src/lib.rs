/// item_nft — creates and burns game-item NFTs (SPL Token-2022, supply = 1).
///
/// Each item is represented as an SPL Token-2022 mint with:
///   • supply = 1 (non-fungible)
///   • decimals = 0
///   • freeze_authority = item_nft_authority PDA  (prevents transfers except via CPI)
///   • mint_authority   = item_nft_authority PDA
///
/// On-chain metadata is stored in an [`ItemMetadata`] PDA so that the game
/// programs can easily look up an item's type and owner without external calls.
///
/// Access-control
/// --------------
///   • `create_item`  — requires `crafting_authority` signer
///   • `burn_item`    — requires `marketplace_authority` signer
use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{
        self,
        MintTo, Burn,
        FreezeAccount, ThawAccount,
        Token2022,
    },
    token_interface::{Mint, TokenAccount},
};

declare_id!("ItmNFT11111111111111111111111111111111111111");

// ─── Item types ───────────────────────────────────────────────────────────────

pub mod item_type {
    /// Шабля козака  (3× Iron + 1× Wood + 1× Leather)
    pub const SABER:    u8 = 0;
    /// Посох старійшини (2× Wood + 1× Gold + 1× Diamond)
    pub const STAFF:    u8 = 1;
    /// Броня характерника (4× Leather + 2× Iron + 1× Gold)  — optional
    pub const ARMOR:    u8 = 2;
    /// Бойовий браслет (4× Iron + 2× Gold + 2× Diamond) — optional
    pub const BRACELET: u8 = 3;
}

pub const NUM_ITEM_TYPES: u8 = 4;

// ─── Program ──────────────────────────────────────────────────────────────────

#[program]
pub mod item_nft {
    use super::*;

    /// Initialises the [`ItemNftConfig`] PDA.
    ///
    /// `crafting_authority`    — PDA of the crafting program.
    /// `marketplace_authority` — PDA of the marketplace program.
    pub fn initialize(
        ctx: Context<InitializeItemNft>,
        crafting_authority: Pubkey,
        marketplace_authority: Pubkey,
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.item_nft_config;
        cfg.admin                 = ctx.accounts.admin.key();
        cfg.crafting_authority    = crafting_authority;
        cfg.marketplace_authority = marketplace_authority;
        cfg.items_created         = 0;
        cfg.bump                  = ctx.bumps.item_nft_config;
        Ok(())
    }

    /// Creates a new item NFT and transfers it to the player.
    ///
    /// Expects a freshly-created Token-2022 mint account (supply = 0, decimals = 0)
    /// whose mint_authority and freeze_authority are both `item_nft_authority`.
    /// After this instruction:
    ///   1. One token is minted to `player_token_account`.
    ///   2. The account is frozen so the token cannot be transferred without CPI.
    ///   3. An [`ItemMetadata`] PDA is created.
    pub fn create_item(
        ctx: Context<CreateItem>,
        item_type: u8,
    ) -> Result<()> {
        require!(item_type < NUM_ITEM_TYPES, ItemNftError::InvalidItemType);

        let cfg = &ctx.accounts.item_nft_config;
        require_keys_eq!(
            ctx.accounts.crafting_authority.key(),
            cfg.crafting_authority,
            ItemNftError::Unauthorised
        );

        // ── PDA signer seeds for item_nft_authority ───────────────────────────
        let seeds: &[&[u8]] = &[b"item_nft_authority", &[ctx.bumps.item_nft_authority]];
        let signer = &[seeds];

        // 1. Mint exactly 1 token
        token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint:      ctx.accounts.item_mint.to_account_info(),
                    to:        ctx.accounts.player_token_account.to_account_info(),
                    authority: ctx.accounts.item_nft_authority.to_account_info(),
                },
                signer,
            ),
            1,
        )?;

        // 2. Freeze the token account to prevent direct transfers
        token_2022::freeze_account(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                FreezeAccount {
                    account:   ctx.accounts.player_token_account.to_account_info(),
                    mint:      ctx.accounts.item_mint.to_account_info(),
                    authority: ctx.accounts.item_nft_authority.to_account_info(),
                },
                signer,
            ),
        )?;

        // 3. Store on-chain metadata
        let meta = &mut ctx.accounts.item_metadata;
        meta.item_type  = item_type;
        meta.owner      = ctx.accounts.player.key();
        meta.mint       = ctx.accounts.item_mint.key();
        meta.bump       = ctx.bumps.item_metadata;

        ctx.accounts.item_nft_config.items_created = ctx
            .accounts.item_nft_config.items_created
            .checked_add(1)
            .ok_or(ItemNftError::Overflow)?;

        emit!(ItemCreated {
            player:    ctx.accounts.player.key(),
            item_type,
            mint:      ctx.accounts.item_mint.key(),
        });
        Ok(())
    }

    /// Burns the item NFT.  Called by the marketplace during a sale.
    ///
    /// Steps:
    ///   1. Thaw the token account.
    ///   2. Burn the 1 token.
    ///   3. Close the [`ItemMetadata`] PDA (lamports returned to player).
    pub fn burn_item(ctx: Context<BurnItem>) -> Result<()> {
        let cfg = &ctx.accounts.item_nft_config;
        require_keys_eq!(
            ctx.accounts.marketplace_authority.key(),
            cfg.marketplace_authority,
            ItemNftError::Unauthorised
        );

        // Verify the metadata points to this mint
        require_keys_eq!(
            ctx.accounts.item_metadata.mint,
            ctx.accounts.item_mint.key(),
            ItemNftError::MintMismatch
        );

        let seeds: &[&[u8]] = &[b"item_nft_authority", &[ctx.bumps.item_nft_authority]];
        let signer = &[seeds];

        // 1. Thaw
        token_2022::thaw_account(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                ThawAccount {
                    account:   ctx.accounts.player_token_account.to_account_info(),
                    mint:      ctx.accounts.item_mint.to_account_info(),
                    authority: ctx.accounts.item_nft_authority.to_account_info(),
                },
                signer,
            ),
        )?;

        // 2. Burn
        token_2022::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint:      ctx.accounts.item_mint.to_account_info(),
                    from:      ctx.accounts.player_token_account.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            1,
        )?;

        emit!(ItemBurned {
            player: ctx.accounts.player.key(),
            mint:   ctx.accounts.item_mint.key(),
        });
        Ok(())
    }
}

// ─── Accounts ─────────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct InitializeItemNft<'info> {
    #[account(
        init,
        payer  = admin,
        space  = ItemNftConfig::LEN,
        seeds  = [b"item_nft_config"],
        bump,
    )]
    pub item_nft_config: Account<'info, ItemNftConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateItem<'info> {
    #[account(mut, seeds = [b"item_nft_config"], bump = item_nft_config.bump)]
    pub item_nft_config: Account<'info, ItemNftConfig>,

    /// This program's authority PDA — signs CPI calls to Token-2022.
    /// CHECK: derived via seeds constraint
    #[account(seeds = [b"item_nft_authority"], bump)]
    pub item_nft_authority: UncheckedAccount<'info>,

    /// Crafting program PDA — must be a signer.
    pub crafting_authority: Signer<'info>,

    /// The player who will receive the item NFT.
    /// CHECK: just stored in metadata
    pub player: UncheckedAccount<'info>,

    /// Freshly created Token-2022 mint (supply=0, decimals=0, mint_authority = item_nft_authority).
    #[account(mut)]
    pub item_mint: InterfaceAccount<'info, Mint>,

    /// Player's token account for this mint.
    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    /// On-chain metadata for this item.
    #[account(
        init,
        payer  = payer,
        space  = ItemMetadata::LEN,
        seeds  = [b"item_metadata", item_mint.key().as_ref()],
        bump,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program:  Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BurnItem<'info> {
    #[account(seeds = [b"item_nft_config"], bump = item_nft_config.bump)]
    pub item_nft_config: Account<'info, ItemNftConfig>,

    /// CHECK: derived via seeds constraint
    #[account(seeds = [b"item_nft_authority"], bump)]
    pub item_nft_authority: UncheckedAccount<'info>,

    /// Marketplace program PDA — must be a signer.
    pub marketplace_authority: Signer<'info>,

    /// The player who owns the item (must sign to approve burn).
    pub player: Signer<'info>,

    #[account(mut)]
    pub item_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close  = player,
        seeds  = [b"item_metadata", item_mint.key().as_ref()],
        bump   = item_metadata.bump,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    pub token_program: Program<'info, Token2022>,
}

// ─── State ────────────────────────────────────────────────────────────────────

#[account]
pub struct ItemNftConfig {
    pub admin: Pubkey,
    pub crafting_authority: Pubkey,
    pub marketplace_authority: Pubkey,
    pub items_created: u64,
    pub bump: u8,
}

impl ItemNftConfig {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 1;
}

/// On-chain metadata for a single item NFT.
#[account]
pub struct ItemMetadata {
    /// Item type constant (see `item_type` module).
    pub item_type: u8,
    /// Current owner of the item.
    pub owner: Pubkey,
    /// The Token-2022 mint address of this item.
    pub mint: Pubkey,
    pub bump: u8,
}

impl ItemMetadata {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 1;
}

// ─── Events ───────────────────────────────────────────────────────────────────

#[event]
pub struct ItemCreated {
    pub player:    Pubkey,
    pub item_type: u8,
    pub mint:      Pubkey,
}

#[event]
pub struct ItemBurned {
    pub player: Pubkey,
    pub mint:   Pubkey,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[error_code]
pub enum ItemNftError {
    #[msg("Invalid item type")]
    InvalidItemType,
    #[msg("Caller is not authorised")]
    Unauthorised,
    #[msg("Mint does not match item metadata")]
    MintMismatch,
    #[msg("Arithmetic overflow")]
    Overflow,
}
