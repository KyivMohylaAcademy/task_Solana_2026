use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token_2022_extensions::{token_metadata_initialize, TokenMetadataInitialize};
use anchor_spl::token_interface::{Mint, TokenInterface};
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_metadata_interface::state::TokenMetadata;

use crate::{
    constants::{
        GAME_CONFIG_SEED, MINT_AUTHORITY_SEED, RESOURCE_COUNT, RESOURCE_MINT_SEED,
        RESOURCE_NAMES, RESOURCE_SYMBOLS,
    },
    error::ResourceManagerError,
    state::GameConfig,
};

/// Accounts required by [`handler`]. Uses
/// `#[instruction(resource_id: u8)]` so that `resource_id` is available in
/// seed expressions — Anchor passes instruction args into constraint
/// evaluation.
#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct InitializeResourceMint<'info> {
    /// The `GameConfig` PDA. Mutated because we write this mint's address
    /// into `resource_mints[resource_id]`.
    #[account(
        mut,
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        has_one = admin,
    )]
    pub game_config: Account<'info, GameConfig>,

    /// The admin — must sign and must match `game_config.admin`.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// Per-resource mint-authority PDA. No private key exists for this
    /// address; only this program can sign as it by supplying the seeds.
    ///
    /// CHECK: seeds uniquely constrain this to the canonical PDA for the
    /// given `resource_id`.
    #[account(
        seeds = [MINT_AUTHORITY_SEED, &[resource_id]],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// The resource's mint account — PDA giving a deterministic address per
    /// `resource_id`.
    ///
    /// The `extensions::metadata_pointer::*` constraints tell Anchor to init
    /// the Token-2022 **MetadataPointer** extension on the mint. The pointer
    /// is self-referential (`metadata_address = mint`) so the actual
    /// metadata will live inside this same mint account via the
    /// **TokenMetadata** extension, written by a follow-up CPI in the
    /// handler below.
    #[account(
        init,
        payer = admin,
        seeds = [RESOURCE_MINT_SEED, &[resource_id]],
        bump,
        mint::decimals = 0,
        mint::authority = mint_authority,
        mint::token_program = token_program,
        extensions::metadata_pointer::authority = mint_authority,
        extensions::metadata_pointer::metadata_address = mint,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// Token-2022 program. Passed via the `TokenInterface` so the same code
    /// would work with classic SPL Token, though our deployment uses 2022.
    pub token_program: Interface<'info, TokenInterface>,

    /// Required by the runtime to create accounts and top up lamports via
    /// CPI.
    pub system_program: Program<'info, System>,
}

/// Create the SPL Token-2022 mint for a single resource, attach the
/// `TokenMetadata` extension (name/symbol), and record the mint address in
/// `GameConfig`.
///
/// The metadata is variable length, so its on-chain footprint isn't known
/// until we serialise it. Flow:
///   1. Build the `TokenMetadata` struct in memory.
///   2. Top up the mint with enough lamports for the larger rent-exempt
///      size (initial mint rent covered the base Mint + MetadataPointer;
///      we need extra for the metadata TLV entry).
///   3. CPI into Token-2022 signed by the `mint_authority` PDA — Token-2022
///      reallocates the mint account to fit and writes the TLV record.
pub fn handler(ctx: Context<InitializeResourceMint>, resource_id: u8) -> Result<()> {
    require!(
        (resource_id as usize) < RESOURCE_COUNT,
        ResourceManagerError::InvalidResourceId
    );

    let game_config = &mut ctx.accounts.game_config;
    let slot = &mut game_config.resource_mints[resource_id as usize];
    require!(
        *slot == Pubkey::default(),
        ResourceManagerError::ResourceMintAlreadyInitialised
    );
    *slot = ctx.accounts.mint.key();

    // Resource display data. URI is left empty — for the homework we only
    // care about name/symbol; external art can be added later via an
    // `update_field` instruction.
    let name = RESOURCE_NAMES[resource_id as usize].to_string();
    let symbol = RESOURCE_SYMBOLS[resource_id as usize].to_string();
    let uri = String::new();

    // Build a TokenMetadata snapshot to measure its borsh-packed size.
    // Token-2022's TLV layout adds 4 bytes of overhead (2 bytes type
    // discriminator + 2 bytes length) per variable-length extension.
    let metadata = TokenMetadata {
        update_authority: OptionalNonZeroPubkey::try_from(Some(
            ctx.accounts.mint_authority.key(),
        ))
        .map_err(|_| ProgramError::InvalidAccountData)?,
        mint: ctx.accounts.mint.key(),
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        additional_metadata: vec![],
    };
    const TLV_OVERHEAD: usize = 4;
    let metadata_data_len = borsh::to_vec(&metadata)
        .map_err(|_| ProgramError::InvalidAccountData)?
        .len();
    let new_size = ctx
        .accounts
        .mint
        .to_account_info()
        .data_len()
        .checked_add(TLV_OVERHEAD)
        .and_then(|x| x.checked_add(metadata_data_len))
        .ok_or(ProgramError::InvalidAccountData)?;

    // Top up the mint account so it stays rent-exempt after Token-2022
    // reallocs it to fit the metadata TLV entry.
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(new_size);
    let current_lamports = ctx.accounts.mint.to_account_info().lamports();
    let lamports_diff = required_lamports.saturating_sub(current_lamports);
    if lamports_diff > 0 {
        transfer(
            CpiContext::new(
                ctx.accounts.system_program.key(),
                Transfer {
                    from: ctx.accounts.admin.to_account_info(),
                    to: ctx.accounts.mint.to_account_info(),
                },
            ),
            lamports_diff,
        )?;
    }

    // Sign as `mint_authority` PDA so Token-2022 accepts the metadata
    // initialisation call. The seeds must match the `mint_authority`
    // account constraint above.
    let mint_authority_bump = ctx.bumps.mint_authority;
    let resource_id_bytes = [resource_id];
    let mint_authority_bump_bytes = [mint_authority_bump];
    let authority_seeds: &[&[u8]] = &[
        MINT_AUTHORITY_SEED,
        &resource_id_bytes,
        &mint_authority_bump_bytes,
    ];
    let signer_seeds: &[&[&[u8]]] = &[authority_seeds];

    token_metadata_initialize(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TokenMetadataInitialize {
                program_id: ctx.accounts.token_program.to_account_info(),
                metadata: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.mint_authority.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        name,
        symbol,
        uri,
    )?;

    msg!(
        "Initialised resource {} with mint {}",
        resource_id,
        ctx.accounts.mint.key()
    );
    Ok(())
}
