use anchor_lang::prelude::*;
use resource_manager::cpi::accounts::MintResource;
use resource_manager::cpi::mint_resource;
use resource_manager::GameConfig;
use solana_keccak_hasher::hash;

declare_id!("JCKmHHAQEmsKEi9DfS7VpiSmmGsMDszfLzGc3baFiSAW");

/// Тривалість кулдауну пошуку у секундах (60 сек).
const SEARCH_COOLDOWN: i64 = 60;

/// Search — програма пошуку ресурсів з он-чейн таймером 60 секунд.
///
/// Гравець може реєструватись та шукати ресурси не частіше ніж раз на 60 секунд.
/// Кожен пошук генерує 3 випадкових ресурси (SPL Token-2022).
#[program]
pub mod search {
    use super::*;

    /// Реєстрація нового гравця.
    ///
    /// Створює PDA-акаунт гравця з seeds = [b"player", player_owner.key()].
    /// last_search_timestamp ініціалізується як 0, що дозволяє перший пошук одразу.
    pub fn register_player(ctx: Context<RegisterPlayer>) -> Result<()> {
        let player_account = &mut ctx.accounts.player_account;
        player_account.owner = ctx.accounts.player_owner.key();
        player_account.last_search_timestamp = 0;
        player_account.bump = ctx.bumps.player_account;

        msg!("Player registered: {}", ctx.accounts.player_owner.key());
        Ok(())
    }

    /// Пошук ресурсів.
    ///
    /// Перевіряє таймер (60 сек), генерує 3 випадкових ресурси та
    /// мінтить їх гравцю через CPI до resource_manager::mint_resource.
    ///
    /// remaining_accounts очікує 12 акаунтів:
    /// - [0..6]: 6 resource mints (Token-2022) у порядку resource_id
    /// - [6..12]: 6 player token accounts (ATA) у порядку resource_id
    pub fn search_resources<'info>(
        ctx: Context<'_, '_, '_, 'info, SearchResources<'info>>,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let player_account = &mut ctx.accounts.player_account;

        // Перевірка кулдауну (60 секунд)
        let elapsed = clock.unix_timestamp - player_account.last_search_timestamp;
        require!(elapsed >= SEARCH_COOLDOWN, SearchError::SearchCooldown);

        // Оновлення таймстампу
        player_account.last_search_timestamp = clock.unix_timestamp;

        // Перевірка що передано достатньо remaining_accounts
        require!(
            ctx.remaining_accounts.len() >= 12,
            SearchError::InsufficientAccounts
        );

        // Генерація 3 псевдо-випадкових resource ID (0-5)
        let seed_data = [
            clock.slot.to_le_bytes().as_ref(),
            clock.unix_timestamp.to_le_bytes().as_ref(),
            ctx.accounts.player_account.key().as_ref(),
        ]
        .concat();
        let hash_result = hash(&seed_data);
        let hash_bytes = hash_result.to_bytes();
        let resource_ids: [u8; 3] = [
            hash_bytes[0] % 6,
            hash_bytes[1] % 6,
            hash_bytes[2] % 6,
        ];

        msg!(
            "Search result: resources [{}, {}, {}]",
            resource_ids[0],
            resource_ids[1],
            resource_ids[2]
        );

        // CPI authority PDA seeds для підпису
        let cpi_bump = ctx.bumps.cpi_authority;
        let cpi_seeds: &[&[u8]] = &[b"cpi_authority", &[cpi_bump]];
        let signer_seeds = &[cpi_seeds];

        // Мінтимо кожен з 3 ресурсів через CPI до resource_manager
        for &resource_id in &resource_ids {
            let resource_mint = ctx.remaining_accounts[resource_id as usize].clone();
            let player_token_account = ctx.remaining_accounts[6 + resource_id as usize].clone();

            let cpi_accounts = MintResource {
                caller_auth: ctx.accounts.cpi_authority.to_account_info(),
                game_config: ctx.accounts.game_config.to_account_info(),
                mint_authority: ctx.accounts.mint_authority.to_account_info(),
                resource_mint: resource_mint,
                player_token_account: player_token_account,
                token_2022_program: ctx.accounts.token_2022_program.to_account_info(),
            };

            let cpi_program = ctx.accounts.resource_manager_program.to_account_info();

            mint_resource(
                CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds),
                resource_id,
                1, // мінтимо по 1 одиниці кожного ресурсу
            )?;
        }

        Ok(())
    }
}

/// Акаунт гравця з он-чейн таймером пошуку.
///
/// PDA seeds: [b"player", owner.key().as_ref()]
#[account]
pub struct PlayerAccount {
    /// Власник акаунту гравця (публічний ключ)
    pub owner: Pubkey,
    /// Таймстамп останнього пошуку (Unix timestamp)
    pub last_search_timestamp: i64,
    /// Bump для PDA
    pub bump: u8,
}

/// Акаунти для реєстрації гравця.
#[derive(Accounts)]
pub struct RegisterPlayer<'info> {
    /// Власник акаунту гравця (підписант та платник)
    #[account(mut)]
    pub player_owner: Signer<'info>,

    /// PDA акаунту гравця
    #[account(
        init,
        payer = player_owner,
        space = 8 + 32 + 8 + 1,
        seeds = [b"player", player_owner.key().as_ref()],
        bump
    )]
    pub player_account: Account<'info, PlayerAccount>,

    /// Системна програма
    pub system_program: Program<'info, System>,
}

/// Акаунти для пошуку ресурсів.
#[derive(Accounts)]
pub struct SearchResources<'info> {
    /// Власник гравця (підписант)
    #[account(mut)]
    pub player_owner: Signer<'info>,

    /// PDA акаунту гравця
    #[account(
        mut,
        seeds = [b"player", player_owner.key().as_ref()],
        bump = player_account.bump,
        constraint = player_account.owner == player_owner.key() @ SearchError::SearchCooldown,
    )]
    pub player_account: Account<'info, PlayerAccount>,

    /// Конфігурація гри (з resource_manager)
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump,
        seeds::program = resource_manager::ID,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// Mint authority PDA від resource_manager
    /// CHECK: PDA resource_manager з seeds [b"mint_authority"]
    #[account(
        seeds = [b"mint_authority"],
        bump = game_config.mint_authority_bump,
        seeds::program = resource_manager::ID,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// CPI authority PDA від search програми
    /// CHECK: PDA цієї програми з seeds [b"cpi_authority"]
    #[account(
        seeds = [b"cpi_authority"],
        bump,
    )]
    pub cpi_authority: UncheckedAccount<'info>,

    /// Програма resource_manager для CPI
    /// CHECK: Перевіряється за program ID
    pub resource_manager_program: UncheckedAccount<'info>,

    /// Token-2022 програма
    /// CHECK: SPL Token-2022 program
    pub token_2022_program: UncheckedAccount<'info>,

    /// Системна програма
    pub system_program: Program<'info, System>,
}

/// Коди помилок для програми search.
#[error_code]
pub enum SearchError {
    /// Пошук ще на кулдауні (60 секунд між пошуками)
    #[msg("Search is on cooldown (60 seconds between searches)")]
    SearchCooldown,
    /// Недостатньо remaining accounts (потрібно 12: 6 мінтів + 6 ATA)
    #[msg("Insufficient remaining accounts (need 12: 6 mints + 6 ATAs)")]
    InsufficientAccounts,
}
