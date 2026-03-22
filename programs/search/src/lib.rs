use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;
use resource_manager::cpi::accounts::MintResource;
use resource_manager::program::ResourceManager;
use resource_manager::{self as resource_mgr, GameConfig};

declare_id!("7LYMjRTUz35XejuLKhawzxVej3saRJ6kHCq6d6kWjeDz");

pub const SEARCH_COOLDOWN: i64 = 60;
pub const RESOURCES_PER_SEARCH: u8 = 3;

#[program]
pub mod search {
    use super::*;

    pub fn register_player(ctx: Context<RegisterPlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player_account;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0;
        player.bump = ctx.bumps.player_account;

        msg!("Search: Registered player {}", ctx.accounts.owner.key());
        Ok(())
    }

    pub fn search_resources<'a>(ctx: Context<'_, '_, 'a, 'a, SearchResources<'a>>) -> Result<()> {
        let player = &mut ctx.accounts.player_account;
        let clock = Clock::get()?;

        let elapsed = clock.unix_timestamp - player.last_search_timestamp;
        require!(elapsed >= SEARCH_COOLDOWN, SearchError::CooldownNotExpired);

        player.last_search_timestamp = clock.unix_timestamp;

        let slot = clock.slot;
        let timestamp = clock.unix_timestamp as u64;
        let player_key = ctx.accounts.owner.key();

        let slot_bytes = slot.to_le_bytes();
        let ts_bytes = timestamp.to_le_bytes();
        let key_bytes = player_key.to_bytes();
        let r0 = slot_bytes[0] ^ ts_bytes[3] ^ key_bytes[0] ^ key_bytes[16];
        let r1 = slot_bytes[2] ^ ts_bytes[1] ^ key_bytes[8] ^ key_bytes[24];
        let r2 = slot_bytes[4] ^ ts_bytes[5] ^ key_bytes[4] ^ key_bytes[20];
        let hash_bytes: [u8; 3] = [r0, r1, r2];

        let resource_ids: [u8; 3] = [
            hash_bytes[0] % 6,
            hash_bytes[1] % 6,
            hash_bytes[2] % 6,
        ];

        msg!(
            "Search: Player {} found resources [{}, {}, {}]",
            player_key,
            resource_ids[0],
            resource_ids[1],
            resource_ids[2]
        );

        let remaining = &ctx.remaining_accounts;
        require!(remaining.len() == 12, SearchError::InvalidAccounts);

        for i in 0..RESOURCES_PER_SEARCH as usize {
            let rid = resource_ids[i] as usize;
            let resource_mint = &remaining[rid];
            let player_token = &remaining[6 + rid];

            let cpi_accounts = MintResource {
                game_config: ctx.accounts.game_config.to_account_info(),
                resource_mint: resource_mint.clone(),
                mint_authority: ctx.accounts.mint_authority.to_account_info(),
                player_token_account: player_token.clone(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };

            let cpi_ctx = CpiContext::new(
                ctx.accounts.resource_manager_program.to_account_info(),
                cpi_accounts,
            );

            resource_mgr::cpi::mint_resource(cpi_ctx, resource_ids[i], 1)?;
        }

        Ok(())
    }
}

#[account]
pub struct Player {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub bump: u8,
}

impl Player {
    pub const LEN: usize = 8 + 32 + 8 + 1;
}

#[derive(Accounts)]
pub struct RegisterPlayer<'info> {
    #[account(
        init,
        payer = owner,
        space = Player::LEN,
        seeds = [b"player", owner.key().as_ref()],
        bump,
    )]
    pub player_account: Account<'info, Player>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SearchResources<'info> {
    #[account(
        mut,
        seeds = [b"player", owner.key().as_ref()],
        bump = player_account.bump,
        constraint = player_account.owner == owner.key() @ SearchError::Unauthorized,
    )]
    pub player_account: Account<'info, Player>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub game_config: Account<'info, GameConfig>,

    /// CHECK: PDA mint authority
    pub mint_authority: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,

    pub resource_manager_program: Program<'info, ResourceManager>,
}

#[error_code]
pub enum SearchError {
    #[msg("Search cooldown has not expired. Wait 60 seconds.")]
    CooldownNotExpired,
    #[msg("Unauthorized: not the player owner.")]
    Unauthorized,
    #[msg("Invalid remaining accounts.")]
    InvalidAccounts,
}
