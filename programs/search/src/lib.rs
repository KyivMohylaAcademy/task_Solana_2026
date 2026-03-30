//! Resource-search gameplay loop with cooldown tracking and reward minting.

use anchor_lang::prelude::*;
use anchor_spl::associated_token::{
    create_idempotent, get_associated_token_address_with_program_id, AssociatedToken, Create,
};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::Mint;
use resource_manager::{program::ResourceManager, GameConfig};
use shared::{
    GameErrorCode, ResourceType, GAME_CONFIG_SEED, PLAYER_SEED, PROGRAM_AUTHORITY_SEED,
    RESOURCE_COUNT, RESOURCE_MANAGER_PROGRAM_ID, SEARCH_COOLDOWN_SECONDS,
};
use solana_sha256_hasher::hashv;

declare_id!("5vrMHniMhyCnZBK5PWTMMF2w886LDc1Kd3GdN17cbPGh");

/// Lets players initialize a search profile and draw random resources.
#[program]
pub mod search {
    use super::*;

    /// Creates the per-wallet `Player` account used to track search cooldowns.
    pub fn init_player(ctx: Context<InitPlayer>) -> Result<()> {
        shared::validate_bootstrap_config()?;

        let player = &mut ctx.accounts.player;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0;
        player.bump = ctx.bumps.player;

        Ok(())
    }

    /// Draws three resources, creates missing ATAs and mints the rewards via CPI.
    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        shared::validate_bootstrap_config()?;
        ctx.accounts.validate_resource_accounts()?;

        let clock = Clock::get()?;
        let current_timestamp = clock.unix_timestamp;
        let last_search_timestamp = ctx.accounts.player.last_search_timestamp;

        require!(
            last_search_timestamp == 0
                || current_timestamp.saturating_sub(last_search_timestamp)
                    >= SEARCH_COOLDOWN_SECONDS,
            GameErrorCode::SearchCooldownActive
        );

        let authority_bump = ctx.bumps.search_authority;
        let signer_seeds: &[&[u8]] = &[PROGRAM_AUTHORITY_SEED, &[authority_bump]];

        for draw_index in 0_u8..3_u8 {
            let resource_type = draw_resource_type(
                &ctx.accounts.owner.key(),
                clock.slot,
                current_timestamp,
                draw_index,
            )?;
            let resource_mint = ctx.accounts.resource_mint_account(resource_type);
            let player_resource_token_account = ctx.accounts.resource_token_account(resource_type);

            create_idempotent(CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                Create {
                    payer: ctx.accounts.owner.to_account_info(),
                    associated_token: player_resource_token_account.clone(),
                    authority: ctx.accounts.owner.to_account_info(),
                    mint: resource_mint.clone(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
            ))?;

            resource_manager::cpi::mint_resource_to_player(
                CpiContext::new_with_signer(
                    ctx.accounts.resource_manager_program.to_account_info(),
                    resource_manager::cpi::accounts::MintResourceToPlayer {
                        player: ctx.accounts.owner.to_account_info(),
                        game_config: ctx.accounts.game_config.to_account_info(),
                        caller_authority: ctx.accounts.search_authority.to_account_info(),
                        program_authority: ctx
                            .accounts
                            .resource_manager_authority
                            .to_account_info(),
                        resource_mint,
                        player_resource_token_account,
                        token_program: ctx.accounts.token_program.to_account_info(),
                    },
                    &[signer_seeds],
                ),
                resource_type as u8,
                1,
            )?;
        }

        ctx.accounts.player.last_search_timestamp = current_timestamp;

        Ok(())
    }

    /// Test-oriented helper that proxies resource minting through the search authority PDA.
    pub fn proxy_mint_resource(
        ctx: Context<ProxyMintResource>,
        resource_type: u8,
        amount: u64,
    ) -> Result<()> {
        let authority_bump = ctx.bumps.search_authority;
        let signer_seeds: &[&[u8]] = &[PROGRAM_AUTHORITY_SEED, &[authority_bump]];

        resource_manager::cpi::mint_resource_to_player(
            CpiContext::new_with_signer(
                ctx.accounts.resource_manager_program.to_account_info(),
                resource_manager::cpi::accounts::MintResourceToPlayer {
                    player: ctx.accounts.owner.to_account_info(),
                    game_config: ctx.accounts.game_config.to_account_info(),
                    caller_authority: ctx.accounts.search_authority.to_account_info(),
                    program_authority: ctx.accounts.resource_manager_authority.to_account_info(),
                    resource_mint: ctx.accounts.resource_mint.to_account_info(),
                    player_resource_token_account: ctx
                        .accounts
                        .player_resource_token_account
                        .to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                &[signer_seeds],
            ),
            resource_type,
            amount,
        )
    }
}

/// Deterministically draws one resource type from slot, timestamp and player entropy.
fn draw_resource_type(
    player_pubkey: &Pubkey,
    slot: u64,
    unix_timestamp: i64,
    draw_index: u8,
) -> Result<ResourceType> {
    let slot_bytes = slot.to_le_bytes();
    let timestamp_bytes = unix_timestamp.to_le_bytes();
    let draw_bytes = [draw_index];
    let hash = hashv(&[
        player_pubkey.as_ref(),
        &slot_bytes,
        &timestamp_bytes,
        &draw_bytes,
    ]);

    let hash_bytes = hash.to_bytes();
    let mut truncated_bytes = [0_u8; 8];
    truncated_bytes.copy_from_slice(&hash_bytes[..8]);

    let resource_index = (u64::from_le_bytes(truncated_bytes) % RESOURCE_COUNT as u64) as usize;
    ResourceType::from_index(resource_index)
}

/// Accounts required to create a player's search profile.
#[derive(Accounts)]
pub struct InitPlayer<'info> {
    /// Wallet that owns the player profile and pays rent.
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        space = 8 + Player::INIT_SPACE,
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump
    )]
    /// PDA storing the owner and last successful search timestamp.
    pub player: Account<'info, Player>,
    /// System program used to create the player account.
    pub system_program: Program<'info, System>,
}

/// Accounts required to execute a search and mint three random resources.
#[derive(Accounts)]
pub struct SearchResources<'info> {
    /// Player wallet authorizing the search and ATA creation.
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == owner.key()
    )]
    /// Player state used to enforce ownership and cooldown.
    pub player: Box<Account<'info, Player>>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump = game_config.bump
    )]
    /// Shared config account holding canonical resource mint addresses.
    pub game_config: Box<Account<'info, GameConfig>>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: this PDA signs the CPI into resource_manager.
    /// Search-program PDA that signs resource-manager mint CPIs.
    pub search_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump
    )]
    /// CHECK: resource_manager validates this PDA again before minting.
    /// Resource-manager PDA that is the actual mint authority for resources.
    pub resource_manager_authority: UncheckedAccount<'info>,
    /// Canonical wood mint.
    #[account(mut)]
    pub wood_mint: Box<InterfaceAccount<'info, Mint>>,
    /// Canonical iron mint.
    #[account(mut)]
    pub iron_mint: Box<InterfaceAccount<'info, Mint>>,
    /// Canonical gold mint.
    #[account(mut)]
    pub gold_mint: Box<InterfaceAccount<'info, Mint>>,
    /// Canonical leather mint.
    #[account(mut)]
    pub leather_mint: Box<InterfaceAccount<'info, Mint>>,
    /// Canonical stone mint.
    #[account(mut)]
    pub stone_mint: Box<InterfaceAccount<'info, Mint>>,
    /// Canonical diamond mint.
    #[account(mut)]
    pub diamond_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    /// CHECK: validated against the expected ATA for the player and wood mint.
    /// Expected owner ATA for wood.
    pub wood_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: validated against the expected ATA for the player and iron mint.
    /// Expected owner ATA for iron.
    pub iron_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: validated against the expected ATA for the player and gold mint.
    /// Expected owner ATA for gold.
    pub gold_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: validated against the expected ATA for the player and leather mint.
    /// Expected owner ATA for leather.
    pub leather_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: validated against the expected ATA for the player and stone mint.
    /// Expected owner ATA for stone.
    pub stone_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: validated against the expected ATA for the player and diamond mint.
    /// Expected owner ATA for diamond.
    pub diamond_token_account: UncheckedAccount<'info>,
    /// CPI interface for resource minting.
    pub resource_manager_program: Program<'info, ResourceManager>,
    /// Associated token program used to create missing resource ATAs.
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// Token-2022 program that owns all resource mints and ATAs.
    pub token_program: Program<'info, Token2022>,
    /// System program used during ATA creation.
    pub system_program: Program<'info, System>,
}

impl<'info> SearchResources<'info> {
    /// Verifies that every mint and ATA account matches the canonical configuration.
    fn validate_resource_accounts(&self) -> Result<()> {
        self.validate_resource_mint(ResourceType::Wood, &self.wood_mint)?;
        self.validate_resource_mint(ResourceType::Iron, &self.iron_mint)?;
        self.validate_resource_mint(ResourceType::Gold, &self.gold_mint)?;
        self.validate_resource_mint(ResourceType::Leather, &self.leather_mint)?;
        self.validate_resource_mint(ResourceType::Stone, &self.stone_mint)?;
        self.validate_resource_mint(ResourceType::Diamond, &self.diamond_mint)?;
        self.validate_resource_token_account(ResourceType::Wood, &self.wood_token_account)?;
        self.validate_resource_token_account(ResourceType::Iron, &self.iron_token_account)?;
        self.validate_resource_token_account(ResourceType::Gold, &self.gold_token_account)?;
        self.validate_resource_token_account(ResourceType::Leather, &self.leather_token_account)?;
        self.validate_resource_token_account(ResourceType::Stone, &self.stone_token_account)?;
        self.validate_resource_token_account(ResourceType::Diamond, &self.diamond_token_account)?;

        Ok(())
    }

    /// Verifies that one mint account matches the expected resource PDA.
    fn validate_resource_mint(
        &self,
        resource_type: ResourceType,
        mint: &InterfaceAccount<'info, Mint>,
    ) -> Result<()> {
        let expected_mint = self.game_config.resource_mints[resource_type.as_index()];
        require_keys_eq!(
            mint.key(),
            expected_mint,
            GameErrorCode::ResourceMintAddressMismatch
        );

        Ok(())
    }

    /// Verifies that one ATA matches the player wallet and resource mint.
    fn validate_resource_token_account(
        &self,
        resource_type: ResourceType,
        token_account: &UncheckedAccount<'info>,
    ) -> Result<()> {
        let expected_token_account = get_associated_token_address_with_program_id(
            &self.owner.key(),
            &self.resource_mint_account(resource_type).key(),
            &self.token_program.key(),
        );
        require_keys_eq!(
            token_account.key(),
            expected_token_account,
            GameErrorCode::InvalidItemTokenAccount
        );

        Ok(())
    }

    /// Returns the mint account for a given resource type.
    fn resource_mint_account(&self, resource_type: ResourceType) -> AccountInfo<'info> {
        match resource_type {
            ResourceType::Wood => self.wood_mint.to_account_info(),
            ResourceType::Iron => self.iron_mint.to_account_info(),
            ResourceType::Gold => self.gold_mint.to_account_info(),
            ResourceType::Leather => self.leather_mint.to_account_info(),
            ResourceType::Stone => self.stone_mint.to_account_info(),
            ResourceType::Diamond => self.diamond_mint.to_account_info(),
        }
    }

    /// Returns the player token account for a given resource type.
    fn resource_token_account(&self, resource_type: ResourceType) -> AccountInfo<'info> {
        match resource_type {
            ResourceType::Wood => self.wood_token_account.to_account_info(),
            ResourceType::Iron => self.iron_token_account.to_account_info(),
            ResourceType::Gold => self.gold_token_account.to_account_info(),
            ResourceType::Leather => self.leather_token_account.to_account_info(),
            ResourceType::Stone => self.stone_token_account.to_account_info(),
            ResourceType::Diamond => self.diamond_token_account.to_account_info(),
        }
    }
}

/// Accounts required by tests to mint resources through the authorized search PDA.
#[derive(Accounts)]
pub struct ProxyMintResource<'info> {
    /// Player wallet that owns the target resource ATA.
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == owner.key()
    )]
    /// Player state proving the owner has been initialized.
    pub player: Account<'info, Player>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump = game_config.bump
    )]
    /// Shared config used by `resource_manager` during mint validation.
    pub game_config: Account<'info, GameConfig>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: this PDA signs the CPI into resource_manager.
    /// Search-program PDA that signs the CPI.
    pub search_authority: UncheckedAccount<'info>,
    #[account(
        seeds = [PROGRAM_AUTHORITY_SEED],
        seeds::program = RESOURCE_MANAGER_PROGRAM_ID,
        bump
    )]
    /// CHECK: resource_manager validates this PDA again before minting.
    /// Resource-manager PDA that owns the mint authority.
    pub resource_manager_authority: UncheckedAccount<'info>,
    /// Resource mint that will be minted to the player.
    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    /// CHECK: resource_manager validates the Token-2022 account ownership and mint.
    /// Player ATA receiving the minted tokens.
    pub player_resource_token_account: UncheckedAccount<'info>,
    /// CPI interface for the resource manager program.
    pub resource_manager_program: Program<'info, ResourceManager>,
    /// Token-2022 program used for minting.
    pub token_program: Program<'info, Token2022>,
}

/// Player account tracking ownership and the latest successful search time.
#[account]
#[derive(InitSpace)]
pub struct Player {
    /// Wallet that owns this player profile.
    pub owner: Pubkey,
    /// Unix timestamp of the last successful search action.
    pub last_search_timestamp: i64,
    /// PDA bump for the player account.
    pub bump: u8,
}
