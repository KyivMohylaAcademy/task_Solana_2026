use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, program::invoke_signed, system_instruction};
use anchor_spl::token_2022::{
    self, Burn, FreezeAccount, MintTo, ThawAccount, Token2022, TransferChecked,
};
use anchor_spl::token_2022_extensions::{metadata_pointer_initialize, MetadataPointerInitialize};
use anchor_spl::token_interface::{Mint as TokenMint, TokenAccount};
use game_common::{
    crafting_id, search_id, CRAFTING_AUTHORITY_SEED, GAME_CONFIG_SEED, ITEM_COUNT,
    RESOURCE_AUTHORITY_SEED, RESOURCE_COUNT, RESOURCE_MINT_SEED, SEARCH_AUTHORITY_SEED,
};
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::state::{AccountState, Mint as SplMint};

declare_id!("BnswUmgoVYBc4kkVbGethzDsAoRE4bGX3p19BJ4RuU43");

#[program]
pub mod resource_manager {
    use super::*;

    /// Creates the single game config PDA that stores the admin and canonical mints.
    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        item_prices: [u64; ITEM_COUNT],
    ) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;
        game_config.admin = ctx.accounts.admin.key();
        game_config.resource_mints = [Pubkey::default(); RESOURCE_COUNT];
        game_config.magic_token_mint = Pubkey::default();
        game_config.item_prices = item_prices;
        game_config.bump = ctx.bumps.game_config;
        Ok(())
    }

    /// Creates one Token-2022 resource mint with the MetadataPointer extension enabled.
    pub fn initialize_resource_mint(
        ctx: Context<InitializeResourceMint>,
        resource_id: u8,
        _name: String,
        _symbol: String,
        _uri: String,
    ) -> Result<()> {
        let resource_index = resource_id as usize;
        require!(
            resource_index < RESOURCE_COUNT,
            ErrorCode::InvalidResourceId
        );
        require!(
            ctx.accounts.game_config.resource_mints[resource_index] == Pubkey::default(),
            ErrorCode::MintAlreadyInitialized
        );
        let mint_len = ExtensionType::try_calculate_account_len::<SplMint>(&[ExtensionType::MetadataPointer])
            .expect("metadata pointer mint layout is valid");
        let rent_lamports = Rent::get()?.minimum_balance(mint_len);
        let game_config_key = ctx.accounts.game_config.key();

        let resource_seed = [resource_id];
        let mint_signer_seeds: &[&[u8]] = &[
            RESOURCE_MINT_SEED,
            game_config_key.as_ref(),
            &resource_seed,
            &[ctx.bumps.mint],
        ];
        invoke_signed(
            &system_instruction::create_account(
                &ctx.accounts.admin.key(),
                &ctx.accounts.mint.key(),
                rent_lamports,
                mint_len as u64,
                &ctx.accounts.token_program.key(),
            ),
            &[
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.mint.to_account_info(),
            ],
            &[mint_signer_seeds],
        )?;

        metadata_pointer_initialize(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MetadataPointerInitialize {
                    token_program_id: ctx.accounts.token_program.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            Some(ctx.accounts.mint_authority.key()),
            Some(ctx.accounts.mint.key()),
        )?;

        let initialize_mint_ix = spl_token_2022::instruction::initialize_mint2(
            &ctx.accounts.token_program.key(),
            &ctx.accounts.mint.key(),
            &ctx.accounts.mint_authority.key(),
            Some(&ctx.accounts.mint_authority.key()),
            0,
        )?;
        invoke(
            &initialize_mint_ix,
            &[
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.mint.to_account_info(),
            ],
        )?;

        ctx.accounts.game_config.resource_mints[resource_index] = ctx.accounts.mint.key();
        Ok(())
    }

    /// Registers the canonical MagicToken mint once the mint exists.
    pub fn register_magic_token_mint(
        ctx: Context<RegisterMagicTokenMint>,
    ) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;
        require!(
            game_config.magic_token_mint == Pubkey::default(),
            ErrorCode::MagicTokenMintAlreadySet
        );
        game_config.magic_token_mint = ctx.accounts.magic_token_mint.key();
        Ok(())
    }

    /// Mints one of the canonical resource tokens to a player's ATA.
    pub fn mint_resource(ctx: Context<MintResource>, resource_id: u8, amount: u64) -> Result<()> {
        let resource_index = resource_id as usize;
        require!(
            resource_index < RESOURCE_COUNT,
            ErrorCode::InvalidResourceId
        );
        require!(
            ctx.accounts.game_config.resource_mints[resource_index] == ctx.accounts.mint.key(),
            ErrorCode::InvalidMint
        );
        require!(amount > 0, ErrorCode::InvalidAmount);
        validate_resource_token_account(&ctx.accounts.destination, ctx.accounts.mint.key())?;
        validate_resource_caller(ctx.accounts.authority.key(), ctx.accounts.game_config.key())?;
        let game_config_key = ctx.accounts.game_config.key();

        let signer_seeds: &[&[u8]] = &[
            RESOURCE_AUTHORITY_SEED,
            game_config_key.as_ref(),
            &[ctx.bumps.mint_authority],
        ];
        thaw_resource_account_if_needed(
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.destination.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_authority.to_account_info(),
            signer_seeds,
            ctx.accounts.destination.state == AccountState::Frozen,
        )?;
        token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
        )?;
        freeze_resource_account_if_needed(
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.destination.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_authority.to_account_info(),
            signer_seeds,
            true,
        )?;
        Ok(())
    }

    /// Burns player resources, but only when the call comes through the crafting program.
    pub fn burn_resource(ctx: Context<BurnResource>, resource_id: u8, amount: u64) -> Result<()> {
        let resource_index = resource_id as usize;
        require!(
            resource_index < RESOURCE_COUNT,
            ErrorCode::InvalidResourceId
        );
        require!(
            ctx.accounts.game_config.resource_mints[resource_index] == ctx.accounts.mint.key(),
            ErrorCode::InvalidMint
        );
        require!(amount > 0, ErrorCode::InvalidAmount);
        require_keys_eq!(
            ctx.accounts.source.owner,
            ctx.accounts.owner.key(),
            ErrorCode::UnauthorizedTokenOwner
        );
        validate_resource_token_account(&ctx.accounts.source, ctx.accounts.mint.key())?;
        validate_crafting_caller(ctx.accounts.authority.key(), ctx.accounts.game_config.key())?;

        let game_config_key = ctx.accounts.game_config.key();
        let signer_seeds: &[&[u8]] = &[
            RESOURCE_AUTHORITY_SEED,
            game_config_key.as_ref(),
            &[ctx.bumps.mint_authority],
        ];
        let should_refreeze = ctx.accounts.source.amount > amount;

        thaw_resource_account_if_needed(
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.source.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_authority.to_account_info(),
            signer_seeds,
            ctx.accounts.source.state == AccountState::Frozen,
        )?;
        token_2022::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.source.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            amount,
        )?;
        freeze_resource_account_if_needed(
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.source.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_authority.to_account_info(),
            signer_seeds,
            should_refreeze,
        )?;
        Ok(())
    }

    /// Transfers resources between players through the program so the freeze guard stays intact.
    pub fn transfer_resource(
        ctx: Context<TransferResource>,
        resource_id: u8,
        amount: u64,
    ) -> Result<()> {
        let resource_index = resource_id as usize;
        require!(
            resource_index < RESOURCE_COUNT,
            ErrorCode::InvalidResourceId
        );
        require!(
            ctx.accounts.game_config.resource_mints[resource_index] == ctx.accounts.mint.key(),
            ErrorCode::InvalidMint
        );
        require!(amount > 0, ErrorCode::InvalidAmount);
        require!(
            ctx.accounts.source.key() != ctx.accounts.destination.key(),
            ErrorCode::SameTokenAccount
        );
        require_keys_eq!(
            ctx.accounts.source.owner,
            ctx.accounts.owner.key(),
            ErrorCode::UnauthorizedTokenOwner
        );
        validate_resource_token_account(&ctx.accounts.source, ctx.accounts.mint.key())?;
        validate_resource_token_account(&ctx.accounts.destination, ctx.accounts.mint.key())?;

        let game_config_key = ctx.accounts.game_config.key();
        let signer_seeds: &[&[u8]] = &[
            RESOURCE_AUTHORITY_SEED,
            game_config_key.as_ref(),
            &[ctx.bumps.mint_authority],
        ];
        let should_refreeze_source = ctx.accounts.source.amount > amount;
        let should_refreeze_destination = true;

        thaw_resource_account_if_needed(
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.source.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_authority.to_account_info(),
            signer_seeds,
            ctx.accounts.source.state == AccountState::Frozen,
        )?;
        thaw_resource_account_if_needed(
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.destination.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_authority.to_account_info(),
            signer_seeds,
            ctx.accounts.destination.state == AccountState::Frozen,
        )?;
        token_2022::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.source.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.destination.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            amount,
            ctx.accounts.mint.decimals,
        )?;
        freeze_resource_account_if_needed(
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.source.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_authority.to_account_info(),
            signer_seeds,
            should_refreeze_source,
        )?;
        freeze_resource_account_if_needed(
            &ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.destination.to_account_info(),
            &ctx.accounts.mint.to_account_info(),
            &ctx.accounts.mint_authority.to_account_info(),
            signer_seeds,
            should_refreeze_destination,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [GAME_CONFIG_SEED],
        bump,
        space = 8 + GameConfig::INIT_SPACE
    )]
    pub game_config: Account<'info, GameConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(resource_id: u8, name: String, symbol: String, uri: String)]
pub struct InitializeResourceMint<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        has_one = admin @ ErrorCode::Unauthorized
    )]
    pub game_config: Account<'info, GameConfig>,
    #[account(
        mut,
        seeds = [RESOURCE_MINT_SEED, game_config.key().as_ref(), &[resource_id]],
        bump
    )]
    /// CHECK: The handler derives and creates this PDA as a Token-2022 mint.
    pub mint: UncheckedAccount<'info>,
    /// CHECK: PDA signer used as the only mint authority for resource tokens.
    #[account(seeds = [RESOURCE_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterMagicTokenMint<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        has_one = admin @ ErrorCode::Unauthorized
    )]
    pub game_config: Account<'info, GameConfig>,
    /// CHECK: The admin registers an already-created MagicToken mint PDA here.
    pub magic_token_mint: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct MintResource<'info> {
    pub authority: Signer<'info>,
    #[account(seeds = [GAME_CONFIG_SEED], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, TokenMint>,
    #[account(mut)]
    pub destination: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: PDA signer derived from the game config and owned by this program.
    #[account(seeds = [RESOURCE_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct BurnResource<'info> {
    pub authority: Signer<'info>,
    pub owner: Signer<'info>,
    #[account(seeds = [GAME_CONFIG_SEED], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, TokenMint>,
    #[account(mut)]
    pub source: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: PDA signer derived from the game config and owned by this program.
    #[account(seeds = [RESOURCE_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct TransferResource<'info> {
    pub owner: Signer<'info>,
    #[account(seeds = [GAME_CONFIG_SEED], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,
    #[account(mut)]
    pub mint: InterfaceAccount<'info, TokenMint>,
    #[account(mut)]
    pub source: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub destination: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: PDA signer derived from the game config and owned by this program.
    #[account(seeds = [RESOURCE_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token2022>,
}

/// Canonical game config shared by every program.
#[account]
#[derive(InitSpace)]
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; RESOURCE_COUNT],
    pub magic_token_mint: Pubkey,
    pub item_prices: [u64; ITEM_COUNT],
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Only the configured admin can call this instruction.")]
    Unauthorized,
    #[msg("Resource id is outside the supported range.")]
    InvalidResourceId,
    #[msg("The mint for this slot already exists.")]
    MintAlreadyInitialized,
    #[msg("The provided mint does not match the config.")]
    InvalidMint,
    #[msg("Amount must be greater than zero.")]
    InvalidAmount,
    #[msg("MagicToken mint is already registered.")]
    MagicTokenMintAlreadySet,
    #[msg("Only crafting or search may mint resources.")]
    UnauthorizedCaller,
    #[msg("Only crafting may burn player resources.")]
    UnauthorizedBurnCaller,
    #[msg("The provided token account does not belong to the expected resource mint.")]
    InvalidTokenAccount,
    #[msg("Only the owner of the resource token account may move or burn it.")]
    UnauthorizedTokenOwner,
    #[msg("Source and destination token accounts must be different.")]
    SameTokenAccount,
}

fn validate_resource_caller(authority: Pubkey, game_config: Pubkey) -> Result<()> {
    let expected_search =
        Pubkey::find_program_address(&[SEARCH_AUTHORITY_SEED, game_config.as_ref()], &search_id())
            .0;
    let expected_crafting = Pubkey::find_program_address(
        &[CRAFTING_AUTHORITY_SEED, game_config.as_ref()],
        &crafting_id(),
    )
    .0;
    require!(
        authority == expected_search || authority == expected_crafting,
        ErrorCode::UnauthorizedCaller
    );
    Ok(())
}

fn validate_crafting_caller(authority: Pubkey, game_config: Pubkey) -> Result<()> {
    let expected_crafting = Pubkey::find_program_address(
        &[CRAFTING_AUTHORITY_SEED, game_config.as_ref()],
        &crafting_id(),
    )
    .0;
    require!(
        authority == expected_crafting,
        ErrorCode::UnauthorizedBurnCaller
    );
    Ok(())
}

fn validate_resource_token_account(
    token_account: &InterfaceAccount<TokenAccount>,
    mint: Pubkey,
) -> Result<()> {
    require_keys_eq!(token_account.mint, mint, ErrorCode::InvalidTokenAccount);
    Ok(())
}

fn thaw_resource_account_if_needed<'info>(
    token_program: &AccountInfo<'info>,
    account: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    signer_seeds: &[&[u8]],
    should_thaw: bool,
) -> Result<()> {
    if !should_thaw {
        return Ok(());
    }
    token_2022::thaw_account(CpiContext::new_with_signer(
        token_program.clone(),
        ThawAccount {
            account: account.clone(),
            mint: mint.clone(),
            authority: authority.clone(),
        },
        &[signer_seeds],
    ))?;
    Ok(())
}

fn freeze_resource_account_if_needed<'info>(
    token_program: &AccountInfo<'info>,
    account: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    signer_seeds: &[&[u8]],
    should_freeze: bool,
) -> Result<()> {
    if !should_freeze {
        return Ok(());
    }
    token_2022::freeze_account(CpiContext::new_with_signer(
        token_program.clone(),
        FreezeAccount {
            account: account.clone(),
            mint: mint.clone(),
            authority: authority.clone(),
        },
        &[signer_seeds],
    ))?;
    Ok(())
}
