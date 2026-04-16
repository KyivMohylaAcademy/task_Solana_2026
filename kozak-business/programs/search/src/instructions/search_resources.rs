use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use resource_manager::constants::{
    RESOURCE_COUNT, RESOURCE_MINT_SEED, SEARCH_AUTHORITY_SEED,
};
use resource_manager::cpi::accounts::MintResource;
use resource_manager::cpi::mint_resource;
use resource_manager::program::ResourceManager;
use resource_manager::state::GameConfig;

use crate::{
    constants::{RESOURCES_PER_SEARCH, SEARCH_COOLDOWN_SECONDS},
    error::SearchError,
    state::Player,
};

/// Number of tokens minted per drawn resource. Each draw mints exactly one.
const TOKENS_PER_DRAW: u64 = 1;

/// Accounts required by [`handler`].
///
/// We list all 6 mints, all 6 mint-authority PDAs and all 6 player ATAs
/// up front. The on-chain RNG picks `RESOURCES_PER_SEARCH` of them; the
/// handler then CPIs into `resource_manager::mint_resource` for each pick.
///
/// ATAs must already exist — the test client creates them via the standard
/// SPL Associated Token Program before calling this instruction. Doing it
/// here would burn most of the compute budget on rent-exempt creations the
/// player will likely never use.
#[derive(Accounts)]
pub struct SearchResources<'info> {
    /// The player record. Mutated to update `last_search_timestamp`.
    #[account(
        mut,
        seeds = [crate::constants::PLAYER_SEED, wallet.key().as_ref()],
        bump = player.bump,
        has_one = wallet,
    )]
    pub player: Account<'info, Player>,

    /// The wallet driving the search. Must sign — both to authorise the
    /// state mutation and to act as the implicit fee payer.
    #[account(mut)]
    pub wallet: Signer<'info>,

    /// PDA derived under THIS program's ID. `mint_resource` accepts only
    /// signers whose seeds match `[SEARCH_AUTHORITY_SEED]` under
    /// `game_config.search_program` — which is us.
    ///
    /// CHECK: Constraint enforced by Anchor via seeds.
    #[account(
        seeds = [SEARCH_AUTHORITY_SEED],
        bump,
    )]
    pub search_authority: UncheckedAccount<'info>,

    /// Resource_manager's GameConfig PDA. Read by `mint_resource` to look
    /// up the registered search program — we have to pass the actual data,
    /// not just the address, so the inner instruction can deserialise it.
    pub game_config: Account<'info, GameConfig>,

    // -------- per-resource accounts: mint + authority + player ATA --------
    // Box<> heap-allocates the deserialized data so these 12 large types
    // don't blow Solana's 4096-byte stack limit inside try_accounts.

    /// CHECK: validated by resource_manager via seeds.
    pub mint_authority_0: UncheckedAccount<'info>,
    #[account(mut, mint::token_program = token_program)]
    pub mint_0: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, token::mint = mint_0, token::authority = wallet, token::token_program = token_program)]
    pub ata_0: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: validated by resource_manager via seeds.
    pub mint_authority_1: UncheckedAccount<'info>,
    #[account(mut, mint::token_program = token_program)]
    pub mint_1: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, token::mint = mint_1, token::authority = wallet, token::token_program = token_program)]
    pub ata_1: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: validated by resource_manager via seeds.
    pub mint_authority_2: UncheckedAccount<'info>,
    #[account(mut, mint::token_program = token_program)]
    pub mint_2: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, token::mint = mint_2, token::authority = wallet, token::token_program = token_program)]
    pub ata_2: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: validated by resource_manager via seeds.
    pub mint_authority_3: UncheckedAccount<'info>,
    #[account(mut, mint::token_program = token_program)]
    pub mint_3: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, token::mint = mint_3, token::authority = wallet, token::token_program = token_program)]
    pub ata_3: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: validated by resource_manager via seeds.
    pub mint_authority_4: UncheckedAccount<'info>,
    #[account(mut, mint::token_program = token_program)]
    pub mint_4: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, token::mint = mint_4, token::authority = wallet, token::token_program = token_program)]
    pub ata_4: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: validated by resource_manager via seeds.
    pub mint_authority_5: UncheckedAccount<'info>,
    #[account(mut, mint::token_program = token_program)]
    pub mint_5: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, token::mint = mint_5, token::authority = wallet, token::token_program = token_program)]
    pub ata_5: Box<InterfaceAccount<'info, TokenAccount>>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub token_program: Interface<'info, TokenInterface>,
}

/// Run a single "search the woods" turn for the player.
///
/// Flow:
///   1. Verify the cooldown has elapsed.
///   2. Verify each supplied mint is the canonical PDA for its resource id.
///   3. Hash `(clock.slot, clock.unix_timestamp, player_key, last_ts)` to
///      get a pseudo-random byte stream; take `RESOURCES_PER_SEARCH` bytes
///      and reduce each mod `RESOURCE_COUNT` to pick resource ids.
///   4. CPI into `resource_manager::mint_resource` once per picked id,
///      signing as the `search_authority` PDA.
///   5. Update `last_search_timestamp`.
///
/// Randomness caveat: this is *pseudo*-random. A validator producing the
/// block can predict and partly influence `slot`, `unix_timestamp` and the
/// inclusion order of transactions, so a determined adversary can bias the
/// outcome. Adequate for a homework demo, never trust it with real value —
/// real games use VRF (e.g. Switchboard) or commit-reveal schemes.
pub fn handler(ctx: Context<SearchResources>) -> Result<()> {
    let clock = Clock::get()?;

    // Cooldown gate. `last_search_timestamp == 0` is the "never searched"
    // sentinel and always passes.
    let elapsed = clock.unix_timestamp - ctx.accounts.player.last_search_timestamp;
    require!(
        ctx.accounts.player.last_search_timestamp == 0 || elapsed >= SEARCH_COOLDOWN_SECONDS,
        SearchError::CooldownNotElapsed
    );

    let resource_manager_pid = ctx.accounts.resource_manager_program.key();

    // Bind each fixed account into a single indexable array so the loop
    // below can dispatch by `resource_id`. The mint/authority/ata for slot
    // `i` MUST correspond to resource id `i` — we verify that next.
    let slots: [(
        AccountInfo<'_>,
        AccountInfo<'_>,
        AccountInfo<'_>,
    ); RESOURCE_COUNT] = [
        (
            ctx.accounts.mint_0.to_account_info(),
            ctx.accounts.mint_authority_0.to_account_info(),
            ctx.accounts.ata_0.to_account_info(),
        ),
        (
            ctx.accounts.mint_1.to_account_info(),
            ctx.accounts.mint_authority_1.to_account_info(),
            ctx.accounts.ata_1.to_account_info(),
        ),
        (
            ctx.accounts.mint_2.to_account_info(),
            ctx.accounts.mint_authority_2.to_account_info(),
            ctx.accounts.ata_2.to_account_info(),
        ),
        (
            ctx.accounts.mint_3.to_account_info(),
            ctx.accounts.mint_authority_3.to_account_info(),
            ctx.accounts.ata_3.to_account_info(),
        ),
        (
            ctx.accounts.mint_4.to_account_info(),
            ctx.accounts.mint_authority_4.to_account_info(),
            ctx.accounts.ata_4.to_account_info(),
        ),
        (
            ctx.accounts.mint_5.to_account_info(),
            ctx.accounts.mint_authority_5.to_account_info(),
            ctx.accounts.ata_5.to_account_info(),
        ),
    ];

    // Validate mint ↔ resource_id binding via the canonical PDA derivation.
    // This is what stops a caller from putting `mint_5` in slot 0 and
    // mining the wrong resource.
    for (i, (mint_ai, _, _)) in slots.iter().enumerate() {
        let resource_id_byte = [i as u8];
        let (expected, _) = Pubkey::find_program_address(
            &[RESOURCE_MINT_SEED, &resource_id_byte],
            &resource_manager_pid,
        );
        require_keys_eq!(*mint_ai.key, expected, SearchError::InvalidResourceMint);
    }

    // Pseudo-random draw using a Knuth LCG seeded from on-chain state.
    // Mixing in the player key ensures different players get different draws
    // even in the same slot. NOT cryptographically secure — use a VRF
    // (e.g. Switchboard) in any real-money context.
    let player_seed = u64::from_le_bytes(
        ctx.accounts.player.key().as_ref()[..8]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?,
    );
    // Combine all volatile inputs into a single 64-bit state for the LCG.
    let mut lcg_state = clock.slot
        ^ (clock.unix_timestamp as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ player_seed
        ^ (ctx.accounts.player.last_search_timestamp as u64).wrapping_mul(0x6c62_272e_07bb_0142);

    // Sign as the search_authority PDA for the inner CPI.
    let search_authority_bump = ctx.bumps.search_authority;
    let search_authority_bump_bytes = [search_authority_bump];
    let search_authority_seeds: &[&[u8]] =
        &[SEARCH_AUTHORITY_SEED, &search_authority_bump_bytes];
    let signer_seeds: &[&[&[u8]]] = &[search_authority_seeds];

    for _i in 0..RESOURCES_PER_SEARCH {
        // Advance the LCG: multiplier and increment from Knuth MMIX.
        lcg_state = lcg_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let resource_id = ((lcg_state >> 33) as usize) % RESOURCE_COUNT;
        let (mint_ai, mint_authority_ai, ata_ai) = &slots[resource_id];

        let cpi_accounts = MintResource {
            game_config: ctx.accounts.game_config.to_account_info(),
            search_authority: ctx.accounts.search_authority.to_account_info(),
            mint_authority: mint_authority_ai.clone(),
            mint: mint_ai.clone(),
            recipient_ata: ata_ai.clone(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.resource_manager_program.key(),
            cpi_accounts,
            signer_seeds,
        );

        mint_resource(cpi_ctx, resource_id as u8, TOKENS_PER_DRAW)?;

        msg!("Mined resource {}", resource_id);
    }

    ctx.accounts.player.last_search_timestamp = clock.unix_timestamp;
    Ok(())
}
