use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::extension::metadata_pointer::instruction::initialize as init_metadata_pointer;
use spl_token_2022::instruction::initialize_mint2;
use spl_token_metadata_interface::instruction::initialize as init_token_metadata;
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;
use std::mem::size_of;

declare_id!("4DPGoF2kGBmnt4ZJTKzB7vdzwMGvDVEBP2pbMDjtbVib");

/// Resource Manager program for managing SPL Token-2022 resource tokens
#[program]
pub mod resource_manager {
    use super::*;

    /// Initializes the game configuration and mint authority PDA
    ///
    /// Creates the GameConfig PDA with seeds [b"game_config"] and the mint_authority PDA
    /// with seeds [b"mint_authority"]. The admin is set to the signer.
    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        item_prices: [u64; 4],
        search_program: Pubkey,
        crafting_program: Pubkey,
        marketplace_program: Pubkey,
    ) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;
        game_config.admin = ctx.accounts.admin.key();
        game_config.resource_mints = [Pubkey::default(); 6];
        game_config.magic_token_mint = Pubkey::default();
        game_config.item_prices = item_prices;
        game_config.search_program = search_program;
        game_config.crafting_program = crafting_program;
        game_config.marketplace_program = marketplace_program;
        game_config.mint_authority_bump = ctx.bumps.mint_authority;
        game_config.bump = ctx.bumps.game_config;

        Ok(())
    }

    /// Creates a new SPL Token-2022 resource mint with MetadataPointer extension
    ///
    /// Only the admin can call this instruction. Creates a mint with on-chain metadata
    /// and stores its pubkey in game_config.resource_mints[resource_id].
    /// The mint authority is the mint_authority PDA.
    pub fn create_resource_mint(
        ctx: Context<CreateResourceMint>,
        resource_id: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        require!(resource_id < 6, GameError::InvalidResourceId);

        let game_config = &mut ctx.accounts.game_config;
        require_eq!(
            game_config.admin,
            ctx.accounts.admin.key(),
            GameError::OnlyAdmin
        );

        // Check if mint already initialized
        require_eq!(
            game_config.resource_mints[resource_id as usize],
            Pubkey::default(),
            GameError::MintAlreadyInitialized
        );

        let mint_authority_key = ctx.accounts.mint_authority.key();
        let resource_mint_key = ctx.accounts.resource_mint.key();

        // Calculate space needed for Token-2022 mint with MetadataPointer extension
        let token_metadata = TokenMetadata {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            ..Default::default()
        };

        let extension_len = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(
            &[ExtensionType::MetadataPointer],
        )
        .map_err(|_| GameError::InvalidResourceId)?;

        let metadata_len = token_metadata
            .get_packed_len()
            .map_err(|_| GameError::InvalidResourceId)?;
        let metadata_tlv_len = metadata_len
            + size_of::<ExtensionType>()
            + size_of::<spl_token_2022::extension::Length>();
        let total_len = extension_len + metadata_tlv_len;
        let lamports = Rent::get()?.minimum_balance(total_len);

        // 1. Create account for the mint
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::create_account(
                &ctx.accounts.admin.key(),
                &resource_mint_key,
                lamports,
                extension_len as u64,
                &spl_token_2022::id(),
            ),
            &[
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.resource_mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // 2. Initialize MetadataPointer extension
        anchor_lang::solana_program::program::invoke(
            &init_metadata_pointer(
                &spl_token_2022::id(),
                &resource_mint_key,
                Some(mint_authority_key),
                Some(resource_mint_key),
            )?,
            &[
                ctx.accounts.resource_mint.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
        )?;

        // 3. Initialize mint
        anchor_lang::solana_program::program::invoke(
            &initialize_mint2(
                &spl_token_2022::id(),
                &resource_mint_key,
                &mint_authority_key,
                Some(&mint_authority_key),
                0, // decimals
            )?,
            &[
                ctx.accounts.resource_mint.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
        )?;

        // 4. Initialize token metadata on the mint
        let mint_authority_bump_seed = [game_config.mint_authority_bump];
        let mint_authority_seeds: &[&[u8]] = &[b"mint_authority", &mint_authority_bump_seed];
        let signer_seeds = [mint_authority_seeds];

        anchor_lang::solana_program::program::invoke_signed(
            &init_token_metadata(
                &spl_token_2022::id(),
                &resource_mint_key,
                &mint_authority_key,
                &resource_mint_key,
                &mint_authority_key,
                name,
                symbol,
                uri,
            ),
            &[
                ctx.accounts.resource_mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.token_2022_program.to_account_info(),
            ],
            &signer_seeds,
        )?;

        // Store the mint pubkey in game_config
        game_config.resource_mints[resource_id as usize] = resource_mint_key;

        Ok(())
    }

    /// Sets the magic token mint in the game configuration
    ///
    /// Only the admin can call this instruction.
    pub fn set_magic_token_mint(ctx: Context<SetMagicTokenMint>, magic_token_mint: Pubkey) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;
        require_eq!(
            game_config.admin,
            ctx.accounts.admin.key(),
            GameError::OnlyAdmin
        );

        game_config.magic_token_mint = magic_token_mint;

        Ok(())
    }

    /// Mints resource tokens to a player account
    ///
    /// CPI-only instruction called by the search program.
    /// Requires caller_auth to be a PDA derived from the search program.
    pub fn mint_resource(
        ctx: Context<MintResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        require!(resource_id < 6, GameError::InvalidResourceId);

        let game_config = &ctx.accounts.game_config;

        // Verify caller_auth is the expected PDA
        let expected_caller_auth =
            Pubkey::find_program_address(&[b"cpi_authority"], &game_config.search_program).0;
        require_eq!(
            ctx.accounts.caller_auth.key(),
            expected_caller_auth,
            GameError::UnauthorizedCaller
        );

        // Verify resource_mint matches
        let resource_mint = game_config.resource_mints[resource_id as usize];
        require_eq!(
            ctx.accounts.resource_mint.key(),
            resource_mint,
            GameError::InvalidResourceId
        );

        // Mint tokens
        let cpi_accounts = token_2022::MintTo {
            mint: ctx.accounts.resource_mint.to_account_info(),
            to: ctx.accounts.player_token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };

        let mint_authority_bump_seed = [game_config.mint_authority_bump];
        let mint_authority_seeds: &[&[u8]] = &[b"mint_authority", &mint_authority_bump_seed];
        let signer_seeds = [mint_authority_seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_2022_program.to_account_info(),
            cpi_accounts,
            &signer_seeds,
        );

        token_2022::mint_to(cpi_ctx, amount)?;

        Ok(())
    }

    /// Burns resource tokens from a player account
    ///
    /// CPI-only instruction called by the crafting program.
    /// Requires caller_auth to be a PDA derived from the crafting program.
    /// Also requires the player to sign as the token owner.
    pub fn burn_resource(
        ctx: Context<BurnResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        require!(resource_id < 6, GameError::InvalidResourceId);

        let game_config = &ctx.accounts.game_config;

        // Verify caller_auth is the expected PDA
        let expected_caller_auth =
            Pubkey::find_program_address(&[b"cpi_authority"], &game_config.crafting_program).0;
        require_eq!(
            ctx.accounts.caller_auth.key(),
            expected_caller_auth,
            GameError::UnauthorizedCaller
        );

        // Verify resource_mint matches
        let resource_mint = game_config.resource_mints[resource_id as usize];
        require_eq!(
            ctx.accounts.resource_mint.key(),
            resource_mint,
            GameError::InvalidResourceId
        );

        // Burn tokens
        let cpi_accounts = token_2022::Burn {
            mint: ctx.accounts.resource_mint.to_account_info(),
            from: ctx.accounts.player_token_account.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_2022_program.to_account_info(), cpi_accounts);

        token_2022::burn(cpi_ctx, amount)?;

        Ok(())
    }
}

/// Game configuration account
///
/// Stores game settings, resource mint pubkeys, and authorized program IDs.
/// Space calculation: 8 (discriminator) + 32 (admin) + 192 (6*32 mints) + 32 (magic_token_mint)
///                   + 32 (item_prices) + 32 (search_program) + 32 (crafting_program)
///                   + 32 (marketplace_program) + 1 (mint_authority_bump) + 1 (bump) = 394 bytes
#[account]
pub struct GameConfig {
    /// The admin authority who can initialize mints and settings
    pub admin: Pubkey,
    /// Array of 6 resource mint pubkeys
    pub resource_mints: [Pubkey; 6],
    /// The magic token mint pubkey
    pub magic_token_mint: Pubkey,
    /// Item prices in magic tokens (4 item types)
    pub item_prices: [u64; 4],
    /// The search program ID (authorized to call mint_resource)
    pub search_program: Pubkey,
    /// The crafting program ID (authorized to call burn_resource)
    pub crafting_program: Pubkey,
    /// The marketplace program ID
    pub marketplace_program: Pubkey,
    /// Bump for the mint_authority PDA
    pub mint_authority_bump: u8,
    /// Bump for the game_config PDA
    pub bump: u8,
}

/// Initialize game instruction accounts
#[derive(Accounts)]
pub struct InitializeGame<'info> {
    /// The admin signer who initializes the game
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The game configuration PDA (seeds = [b"game_config"])
    #[account(
        init,
        payer = admin,
        space = 8 + 394,
        seeds = [b"game_config"],
        bump
    )]
    pub game_config: Account<'info, GameConfig>,

    /// The mint authority PDA (seeds = [b"mint_authority"])
    /// CHECK: This is a PDA that will be the mint authority for all resource tokens
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}

/// Create resource mint instruction accounts
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct CreateResourceMint<'info> {
    /// The admin signer
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The game configuration account
    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    /// The mint authority PDA
    /// CHECK: This is a PDA that will be the mint authority
    #[account(
        seeds = [b"mint_authority"],
        bump = game_config.mint_authority_bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// The new resource mint account (a new Signer keypair, not a PDA)
    #[account(mut, signer)]
    pub resource_mint: Signer<'info>,

    /// System program for account creation
    pub system_program: Program<'info, System>,

    /// Token-2022 program
    pub token_2022_program: Program<'info, token_2022::Token2022>,
}

/// Set magic token mint instruction accounts
#[derive(Accounts)]
pub struct SetMagicTokenMint<'info> {
    /// The admin signer
    pub admin: Signer<'info>,

    /// The game configuration account
    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,
}

/// Mint resource instruction accounts
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct MintResource<'info> {
    /// The CPI caller authority (PDA from search program)
    pub caller_auth: Signer<'info>,

    /// The game configuration account
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    /// The mint authority PDA
    /// CHECK: This is a PDA that will sign the mint transaction
    #[account(
        seeds = [b"mint_authority"],
        bump = game_config.mint_authority_bump
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// The resource mint account
    /// CHECK: Validated by game_config.resource_mints[resource_id]
    #[account(mut)]
    pub resource_mint: UncheckedAccount<'info>,

    /// The player's token account
    /// CHECK: Validated by token_2022_program
    #[account(mut)]
    pub player_token_account: UncheckedAccount<'info>,

    /// Token-2022 program
    pub token_2022_program: Program<'info, token_2022::Token2022>,
}

/// Burn resource instruction accounts
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct BurnResource<'info> {
    /// The player signer (token owner)
    #[account(mut)]
    pub player: Signer<'info>,

    /// The CPI caller authority (PDA from crafting program)
    pub caller_auth: Signer<'info>,

    /// The game configuration account
    #[account(
        seeds = [b"game_config"],
        bump = game_config.bump
    )]
    pub game_config: Account<'info, GameConfig>,

    /// The resource mint account
    /// CHECK: Validated by game_config.resource_mints[resource_id]
    #[account(mut)]
    pub resource_mint: UncheckedAccount<'info>,

    /// The player's token account
    /// CHECK: Validated by token_2022_program
    #[account(mut)]
    pub player_token_account: UncheckedAccount<'info>,

    /// Token-2022 program
    pub token_2022_program: Program<'info, token_2022::Token2022>,
}

/// Error codes for the resource_manager program
#[error_code]
pub enum GameError {
    #[msg("Invalid resource ID (must be 0-5)")]
    InvalidResourceId,
    #[msg("Unauthorized caller")]
    UnauthorizedCaller,
    #[msg("Only admin can perform this action")]
    OnlyAdmin,
    #[msg("Resource mint already initialized")]
    MintAlreadyInitialized,
}

// CPI модуль генерується автоматично Anchor при компіляції з features = ["cpi"]
