use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, MintTo, Token2022};

declare_id!("7qyvBgEsWYpP5UZKhctCA2C6HuVDBFo4DJH6V2P96rPx");

pub const SEARCH_COOLDOWN: i64 = 60; // секунд

#[program]
pub mod search_program {
    use super::*;

    /// Реєструє нового гравця — створює Player PDA
    pub fn register_player(ctx: Context<RegisterPlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0;
        player.bump = ctx.bumps.player;
        msg!("Player registered: {}", ctx.accounts.owner.key());
        Ok(())
    }

    /// Гравець шукає ресурси — раз на 60 секунд отримує 3 випадкових ресурси
    pub fn search_resources<'a>(ctx: Context<'a, SearchResources<'a>>) -> Result<()> {
        let clock = Clock::get()?;
        let now = clock.unix_timestamp;
        let player = &mut ctx.accounts.player;

        // Перевірка таймера
        require!(
            now - player.last_search_timestamp >= SEARCH_COOLDOWN,
            SearchError::CooldownNotExpired
        );

        // Псевдовипадковість через слот + timestamp + pubkey
        let seed = clock.slot ^ (now as u64) ^ (player.owner.to_bytes()[0] as u64);

        // Генеруємо 3 ресурси (індекси 0-5)
        let r1 = (seed % 6) as usize;
        let r2 = ((seed >> 8) % 6) as usize;
        let r3 = ((seed >> 16) % 6) as usize;

        let resources = [r1, r2, r3];

        // Мінтимо кожен ресурс
        let game_config_bump = ctx.accounts.game_config.bump;
        let seeds_config: &[&[u8]] = &[b"game_config", &[game_config_bump]];
        let signer_seeds = &[seeds_config];

        for &resource_idx in &resources {
            let mint_account = &ctx.remaining_accounts[resource_idx];
            let player_ata = &ctx.remaining_accounts[6 + resource_idx];

            token_2022::mint_to(
                CpiContext::new_with_signer(
                    anchor_spl::token_2022::ID,
                    MintTo {
                        mint: mint_account.to_account_info(),
                        to: player_ata.to_account_info(),
                        authority: ctx.accounts.game_config.to_account_info(),
                    },
                    signer_seeds,
                ),
                1,
            )?;
        }

        // Оновлюємо timestamp
        player.last_search_timestamp = now;
        msg!("Found resources: {}, {}, {}", r1, r2, r3);
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

/// GameConfig — імпортуємо структуру з resource_manager
#[account]
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; 6],
    pub magic_token_mint: Pubkey,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct RegisterPlayer<'info> {
    #[account(
        init,
        payer = owner,
        space = Player::LEN,
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
        mut,
        seeds = [b"player", owner.key().as_ref()],
        bump = player.bump,
        has_one = owner
    )]
    pub player: Account<'info, Player>,

    /// CHECK: game config PDA з resource_manager
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager_program.key()
    )]
    pub game_config: Account<'info, GameConfig>,

    pub owner: Signer<'info>,

    /// CHECK: resource_manager program
    pub resource_manager_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token2022>,
    // remaining_accounts: [mint_0..mint_5, ata_0..ata_5]
}

#[error_code]
pub enum SearchError {
    #[msg("Таймер ще не вийшов. Зачекай 60 секунд.")]
    CooldownNotExpired,
}