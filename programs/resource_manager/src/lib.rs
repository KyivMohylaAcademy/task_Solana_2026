/// resource_manager — manages the 6 SPL Token-2022 resource mints and authorised
/// minting / burning for the "Козацький бізнес" game on Solana.
///
/// Access-control model
/// --------------------
/// Two cross-program authorities are stored in [`GameConfig`]:
///   • `search_authority`  — PDA of the *search* program (`seeds = [b"search_authority"]`)
///   • `crafting_authority` — PDA of the *crafting* program (`seeds = [b"crafting_authority"]`)
///
/// `mint_resources` requires `search_authority` OR `crafting_authority` to sign.
/// `burn_resources` requires `crafting_authority` to sign.
/// This means only the search / crafting programs (who can sign with their own PDAs)
/// are able to mint or burn resource tokens.
use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{
        self, MintTo, Burn, Token2022,
        spl_token_2022::{
            extension::{
                metadata_pointer::MetadataPointer,
                ExtensionType,
            },
            state::Mint as SplMint,
        },
    },
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use spl_token_2022::extension::StateWithExtensions;

declare_id!("ResMgr1111111111111111111111111111111111111");

// ─── Constants ────────────────────────────────────────────────────────────────

/// Number of resource types in the game.
pub const NUM_RESOURCES: usize = 6;

/// Resource type identifiers.
pub mod resource_type {
    pub const WOOD: u8     = 0;
    pub const IRON: u8     = 1;
    pub const GOLD: u8     = 2;
    pub const LEATHER: u8  = 3;
    pub const STONE: u8    = 4;
    pub const DIAMOND: u8  = 5;
}

// ─── Program ──────────────────────────────────────────────────────────────────

#[program]
pub mod resource_manager {
    use super::*;

    /// Initialises the global [`GameConfig`] account.
    ///
    /// Must be called once by the game admin before any other instruction.
    /// `item_prices` specifies the MagicToken reward for selling each item type:
    ///   [saber, staff, armor, bracelet].
    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        search_authority: Pubkey,
        crafting_authority: Pubkey,
        marketplace_authority: Pubkey,
        item_prices: [u64; 4],
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.game_config;
        cfg.admin                 = ctx.accounts.admin.key();
        cfg.resource_mints        = [Pubkey::default(); NUM_RESOURCES];
        cfg.magic_token_mint      = Pubkey::default();
        cfg.search_authority      = search_authority;
        cfg.crafting_authority    = crafting_authority;
        cfg.marketplace_authority = marketplace_authority;
        cfg.item_prices           = item_prices;
        cfg.bump                  = ctx.bumps.game_config;
        Ok(())
    }

    /// Registers a resource mint address in [`GameConfig`].
    ///
    /// The admin calls this after creating each Token-2022 mint client-side
    /// (or via a separate initialisation script) to link it to the config.
    pub fn register_resource_mint(
        ctx: Context<RegisterResourceMint>,
        resource_type: u8,
    ) -> Result<()> {
        require!(resource_type < NUM_RESOURCES as u8, ResourceManagerError::InvalidResourceType);
        let cfg = &mut ctx.accounts.game_config;
        require_keys_eq!(cfg.admin, ctx.accounts.admin.key(), ResourceManagerError::Unauthorised);
        cfg.resource_mints[resource_type as usize] = ctx.accounts.resource_mint.key();
        Ok(())
    }

    /// Registers the MagicToken mint in [`GameConfig`].
    pub fn register_magic_token_mint(ctx: Context<RegisterMagicTokenMint>) -> Result<()> {
        let cfg = &mut ctx.accounts.game_config;
        require_keys_eq!(cfg.admin, ctx.accounts.admin.key(), ResourceManagerError::Unauthorised);
        cfg.magic_token_mint = ctx.accounts.magic_token_mint.key();
        Ok(())
    }

    /// Mints `amount` tokens of `resource_type` to `player_token_account`.
    ///
    /// Caller must provide `authority` which must be a signer AND must match
    /// either `game_config.search_authority` or `game_config.crafting_authority`.
    pub fn mint_resources(
        ctx: Context<MintResources>,
        resource_type: u8,
        amount: u64,
    ) -> Result<()> {
        require!(resource_type < NUM_RESOURCES as u8, ResourceManagerError::InvalidResourceType);
        require!(amount > 0, ResourceManagerError::ZeroAmount);

        let cfg = &ctx.accounts.game_config;

        // ── Access control: authority must be search or crafting ──────────────
        let authority_key = ctx.accounts.authority.key();
        require!(
            authority_key == cfg.search_authority || authority_key == cfg.crafting_authority,
            ResourceManagerError::Unauthorised
        );

        // ── Verify the mint matches the registered resource mint ──────────────
        require_keys_eq!(
            ctx.accounts.resource_mint.key(),
            cfg.resource_mints[resource_type as usize],
            ResourceManagerError::MintMismatch
        );

        // ── Mint via resource_manager's own mint_authority PDA ────────────────
        let seeds: &[&[u8]] = &[b"mint_authority", &[ctx.bumps.mint_authority]];
        let signer_seeds = &[seeds];

        token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint:      ctx.accounts.resource_mint.to_account_info(),
                    to:        ctx.accounts.player_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        emit!(ResourcesMinted {
            player:        ctx.accounts.player_token_account.owner,
            resource_type,
            amount,
        });
        Ok(())
    }

    /// Burns `amount` tokens of `resource_type` from `player_token_account`.
    ///
    /// Caller must provide `authority` which must be a signer AND must match
    /// `game_config.crafting_authority`.
    pub fn burn_resources(
        ctx: Context<BurnResources>,
        resource_type: u8,
        amount: u64,
    ) -> Result<()> {
        require!(resource_type < NUM_RESOURCES as u8, ResourceManagerError::InvalidResourceType);
        require!(amount > 0, ResourceManagerError::ZeroAmount);

        let cfg = &ctx.accounts.game_config;

        // ── Access control: only crafting may burn ────────────────────────────
        require_keys_eq!(
            ctx.accounts.authority.key(),
            cfg.crafting_authority,
            ResourceManagerError::Unauthorised
        );

        // ── Verify mint ───────────────────────────────────────────────────────
        require_keys_eq!(
            ctx.accounts.resource_mint.key(),
            cfg.resource_mints[resource_type as usize],
            ResourceManagerError::MintMismatch
        );

        token_2022::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint:      ctx.accounts.resource_mint.to_account_info(),
                    from:      ctx.accounts.player_token_account.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            amount,
        )?;

        emit!(ResourcesBurned {
            player:        ctx.accounts.player.key(),
            resource_type,
            amount,
        });
        Ok(())
    }
}

// ─── Accounts ─────────────────────────────────────────────────────────────────

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(
        init,
        payer  = admin,
        space  = GameConfig::LEN,
        seeds  = [b"game_config"],
        bump,
    )]
    pub game_config: Account<'info, GameConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterResourceMint<'info> {
    #[account(mut, seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,

    pub admin: Signer<'info>,

    /// CHECK: validated by storing in game_config
    pub resource_mint: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct RegisterMagicTokenMint<'info> {
    #[account(mut, seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,

    pub admin: Signer<'info>,

    /// CHECK: validated by storing in game_config
    pub magic_token_mint: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct MintResources<'info> {
    #[account(seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,

    /// PDA of this program — is the mint authority for all resource mints.
    /// CHECK: derived via seeds constraint
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,

    /// Must be either search_authority or crafting_authority — verified in handler.
    pub authority: Signer<'info>,

    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct BurnResources<'info> {
    #[account(seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,

    /// Crafting authority — verified in handler.
    pub authority: Signer<'info>,

    /// The actual player who owns the tokens (must sign to allow burn).
    pub player: Signer<'info>,

    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub player_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

// ─── State ────────────────────────────────────────────────────────────────────

/// Global game configuration.  Derived via `seeds = [b"game_config"]`.
#[account]
pub struct GameConfig {
    /// Game administrator.
    pub admin: Pubkey,
    /// Mint addresses for the 6 resource tokens (indexed by `resource_type`).
    pub resource_mints: [Pubkey; 6],
    /// Mint address of the MagicToken.
    pub magic_token_mint: Pubkey,
    /// PDA of the search program — authorised to mint resources.
    pub search_authority: Pubkey,
    /// PDA of the crafting program — authorised to mint & burn resources.
    pub crafting_authority: Pubkey,
    /// PDA of the marketplace program — authorised to mint MagicToken.
    pub marketplace_authority: Pubkey,
    /// MagicToken reward per item sale: [saber, staff, armor, bracelet].
    pub item_prices: [u64; 4],
    pub bump: u8,
}

impl GameConfig {
    pub const LEN: usize = 8   // discriminator
        + 32               // admin
        + 32 * 6           // resource_mints
        + 32               // magic_token_mint
        + 32               // search_authority
        + 32               // crafting_authority
        + 32               // marketplace_authority
        + 8 * 4            // item_prices
        + 1;               // bump
}

// ─── Events ───────────────────────────────────────────────────────────────────

#[event]
pub struct ResourcesMinted {
    pub player:        Pubkey,
    pub resource_type: u8,
    pub amount:        u64,
}

#[event]
pub struct ResourcesBurned {
    pub player:        Pubkey,
    pub resource_type: u8,
    pub amount:        u64,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[error_code]
pub enum ResourceManagerError {
    #[msg("Invalid resource type — must be 0-5")]
    InvalidResourceType,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("Caller is not authorised to perform this action")]
    Unauthorised,
    #[msg("Provided mint does not match the registered resource mint")]
    MintMismatch,
}
