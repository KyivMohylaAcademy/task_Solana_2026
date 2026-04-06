use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022, MintTo};

declare_id!("3v5wDujpABMtXCgN6F6YaG5z2LacM2FWNpj31qmgTzjN");

/// Search cooldown period in seconds
const SEARCH_COOLDOWN: i64 = 60;

#[program]
pub mod search {
    use super::*;

    /// Initialize the search program
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.bump = ctx.bumps.config;
        
        msg!("Search program initialized");
        Ok(())
    }

    /// Initialize a player's search account
    pub fn init_player(ctx: Context<InitPlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0; // Allow immediate first search
        player.total_searches = 0;
        player.bump = ctx.bumps.player;
        
        msg!("Player search account initialized for: {}", ctx.accounts.owner.key());
        Ok(())
    }

    /// Search for resources (generates 3 random resources)
    /// Can only be called once per 60 seconds per player
    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Check cooldown
        let time_since_last_search = current_time - player.last_search_timestamp;
        require!(
            time_since_last_search >= SEARCH_COOLDOWN || player.last_search_timestamp == 0,
            ErrorCode::SearchCooldownActive
        );

        // Generate 3 pseudo-random resources
        // Using slot and player's search count for randomness
        let slot = clock.slot;
        let seed = slot
            .wrapping_add(player.total_searches)
            .wrapping_add(current_time as u64);

        let resource1 = (seed % 6) as u8;
        let resource2 = ((seed / 7) % 6) as u8;
        let resource3 = ((seed / 13) % 6) as u8;

        // Update player state
        player.last_search_timestamp = current_time;
        player.total_searches += 1;

        msg!(
            "Player {} found resources: [{}, {}, {}]",
            ctx.accounts.owner.key(),
            resource1,
            resource2,
            resource3
        );

        // Note: Actual minting would be done via CPI to resource_manager
        // For this implementation, we'll emit the resource IDs
        // The client/test will handle the actual minting

        Ok(())
    }

    /// Mint a resource token (internal helper, would use CPI in production)
    pub fn mint_resource_token(
        ctx: Context<MintResourceToken>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        require!(resource_id < 6, ErrorCode::InvalidResourceId);

        let config_seeds: &[&[u8]] = &[
            b"mint_authority",
            &[ctx.accounts.config.bump],
        ];
        let signer = &[&config_seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token_2022::mint_to(cpi_ctx, amount)?;

        msg!("Minted {} units of resource {} to player", amount, resource_id);
        Ok(())
    }

    /// Get time until next search is available
    pub fn get_cooldown_remaining(ctx: Context<GetPlayerInfo>) -> Result<i64> {
        let player = &ctx.accounts.player;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;
        
        let time_since_last = current_time - player.last_search_timestamp;
        let remaining = SEARCH_COOLDOWN - time_since_last;
        
        msg!("Cooldown remaining: {} seconds", remaining.max(0));
        Ok(remaining.max(0))
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + SearchConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, SearchConfig>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitPlayer<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Player::INIT_SPACE,
        seeds = [b"player", owner.key().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SearchResources<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, SearchConfig>,
    
    #[account(
        mut,
        seeds = [b"player", owner.key().as_ref()],
        bump = player.bump,
        constraint = player.owner == owner.key() @ ErrorCode::Unauthorized
    )]
    pub player: Account<'info, Player>,
    
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct MintResourceToken<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, SearchConfig>,
    
    /// CHECK: PDA authority for minting
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    pub mint_authority: AccountInfo<'info>,
    
    /// CHECK: Resource mint
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    
    /// CHECK: Player's token account
    #[account(mut)]
    pub token_account: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct GetPlayerInfo<'info> {
    #[account(
        seeds = [b"player", owner.key().as_ref()],
        bump = player.bump
    )]
    pub player: Account<'info, Player>,
    
    pub owner: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct SearchConfig {
    pub admin: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Player {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub total_searches: u64,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Search cooldown active. Wait 60 seconds between searches")]
    SearchCooldownActive,
    #[msg("Invalid resource ID (must be 0-5)")]
    InvalidResourceId,
    #[msg("Unauthorized")]
    Unauthorized,
}
