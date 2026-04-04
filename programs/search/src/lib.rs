use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint as TokenMint, TokenAccount};
use game_common::{GAME_CONFIG_SEED, PLAYER_SEED, RESOURCE_COUNT, SEARCH_AUTHORITY_SEED};

declare_id!("7yPJgKSZYcUCPgrEBmcQ7z86Frz57H6bsU5hBycStgp9");

#[program]
pub mod search {
    use super::*;

    /// Creates the player PDA that tracks the last successful search.
    pub fn initialize_player(ctx: Context<InitializePlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0;
        player.bump = ctx.bumps.player;
        Ok(())
    }

    /// Generates three resource drops once every 60 seconds.
    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.player.owner,
            ctx.accounts.owner.key(),
            ErrorCode::Unauthorized
        );

        let now = Clock::get()?.unix_timestamp;
        if ctx.accounts.player.last_search_timestamp != 0 {
            require!(
                now - ctx.accounts.player.last_search_timestamp >= 60,
                ErrorCode::SearchCooldownActive
            );
        }

        validate_resource_mints(&ctx.accounts.game_config, &[
            ctx.accounts.wood_mint.key(),
            ctx.accounts.iron_mint.key(),
            ctx.accounts.gold_mint.key(),
            ctx.accounts.leather_mint.key(),
            ctx.accounts.stone_mint.key(),
            ctx.accounts.diamond_mint.key(),
        ])?;

        let drops = roll_resources(ctx.accounts.owner.key(), ctx.accounts.player.last_search_timestamp, Clock::get()?.slot, now);
        let game_config_key = ctx.accounts.game_config.key();
        let search_seeds: &[&[u8]] = &[
            SEARCH_AUTHORITY_SEED,
            game_config_key.as_ref(),
            &[ctx.bumps.search_authority],
        ];

        for resource_id in drops {
            let (mint_info, token_info) = ctx.accounts.resource_accounts(resource_id)?;
            resource_manager::cpi::mint_resource(
                CpiContext::new_with_signer(
                    ctx.accounts.resource_manager_program.to_account_info(),
                    resource_manager::cpi::accounts::MintResource {
                        authority: ctx.accounts.search_authority.to_account_info(),
                        game_config: ctx.accounts.game_config.to_account_info(),
                        mint: mint_info,
                        destination: token_info,
                        mint_authority: ctx.accounts.resource_authority.to_account_info(),
                        token_program: ctx.accounts.token_program.to_account_info(),
                    },
                    &[search_seeds],
                ),
                resource_id,
                1,
            )?;
        }

        ctx.accounts.player.last_search_timestamp = now;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePlayer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump,
        space = 8 + Player::INIT_SPACE
    )]
    pub player: Account<'info, Player>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SearchResources<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump = player.bump
    )]
    pub player: Account<'info, Player>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID
    )]
    pub game_config: Account<'info, resource_manager::GameConfig>,
    /// CHECK: PDA signer that the search program uses when minting resources through CPI.
    #[account(seeds = [SEARCH_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub search_authority: UncheckedAccount<'info>,
    /// CHECK: PDA owned by resource_manager and used as the resource mint authority in CPI.
    pub resource_authority: UncheckedAccount<'info>,
    pub resource_manager_program: Program<'info, resource_manager::program::ResourceManager>,
    #[account(mut)]
    pub wood_mint: Box<InterfaceAccount<'info, TokenMint>>,
    #[account(mut)]
    pub iron_mint: Box<InterfaceAccount<'info, TokenMint>>,
    #[account(mut)]
    pub gold_mint: Box<InterfaceAccount<'info, TokenMint>>,
    #[account(mut)]
    pub leather_mint: Box<InterfaceAccount<'info, TokenMint>>,
    #[account(mut)]
    pub stone_mint: Box<InterfaceAccount<'info, TokenMint>>,
    #[account(mut)]
    pub diamond_mint: Box<InterfaceAccount<'info, TokenMint>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = wood_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub wood_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = iron_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub iron_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = gold_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub gold_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = leather_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub leather_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = stone_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub stone_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = diamond_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program
    )]
    pub diamond_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> SearchResources<'info> {
    fn resource_accounts(
        &self,
        resource_id: u8,
    ) -> Result<(AccountInfo<'info>, AccountInfo<'info>)> {
        let pairs = [
            (
                self.wood_mint.to_account_info(),
                self.wood_account.to_account_info(),
            ),
            (
                self.iron_mint.to_account_info(),
                self.iron_account.to_account_info(),
            ),
            (
                self.gold_mint.to_account_info(),
                self.gold_account.to_account_info(),
            ),
            (
                self.leather_mint.to_account_info(),
                self.leather_account.to_account_info(),
            ),
            (
                self.stone_mint.to_account_info(),
                self.stone_account.to_account_info(),
            ),
            (
                self.diamond_mint.to_account_info(),
                self.diamond_account.to_account_info(),
            ),
        ];
        let index = resource_id as usize;
        debug_assert!(index < RESOURCE_COUNT);
        Ok(pairs[index].clone())
    }
}

/// PDA that throttles search frequency for a player.
#[account]
#[derive(InitSpace)]
pub struct Player {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Only the player who owns this PDA may search.")]
    Unauthorized,
    #[msg("Search can be used once every 60 seconds.")]
    SearchCooldownActive,
    #[msg("Resource mint list does not match the game config.")]
    InvalidMintConfiguration,
    #[msg("Resource id is outside the supported range.")]
    InvalidResourceId,
}

fn validate_resource_mints(
    game_config: &resource_manager::GameConfig,
    provided: &[Pubkey; RESOURCE_COUNT],
) -> Result<()> {
    require!(
        game_config.resource_mints == *provided,
        ErrorCode::InvalidMintConfiguration
    );
    Ok(())
}

fn roll_resources(owner: Pubkey, last_search_timestamp: i64, slot: u64, now: i64) -> [u8; 3] {
    let owner_bytes = owner.to_bytes();
    let mut drops = [0u8; 3];
    let time_seed = (last_search_timestamp.max(0) as u64) ^ (now.max(0) as u64) ^ slot;
    for (index, drop) in drops.iter_mut().enumerate() {
        let a = owner_bytes[(slot as usize + index * 5) % owner_bytes.len()];
        let b = owner_bytes[(slot as usize + index * 11 + 7) % owner_bytes.len()];
        let c = ((time_seed >> (index * 8)) as u8).wrapping_add(index as u8 * 17);
        *drop = a.wrapping_add(b).wrapping_add(c) % (RESOURCE_COUNT as u8);
    }
    drops
}
