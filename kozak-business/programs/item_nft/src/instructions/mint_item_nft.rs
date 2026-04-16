use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, mpl_token_metadata,
        CreateMasterEditionV3, CreateMetadataAccountsV3, Metadata,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

use crate::constants::{ITEM_CONFIG_SEED, NFT_MINT_AUTHORITY_SEED};
use crate::state::ItemConfig;

/// Accounts for [`handler`]. The `mint` is a freshly generated keypair from
/// the caller — it must sign the transaction so Anchor can `init` it.
#[derive(Accounts)]
pub struct MintItemNft<'info> {
    /// The `ItemConfig` PDA — read-only here, just used for authority validation.
    #[account(seeds = [ITEM_CONFIG_SEED], bump = item_config.bump)]
    pub item_config: Account<'info, ItemConfig>,

    /// Program-wide mint-authority PDA. This program signs as it for all
    /// three Metaplex CPIs (metadata + master_edition) and the token mint_to.
    /// After CreateMasterEdition, Metaplex reassigns the mint authority to the
    /// edition PDA — nobody can mint more after that.
    ///
    /// CHECK: Constraint enforced by Anchor via seeds.
    #[account(seeds = [NFT_MINT_AUTHORITY_SEED], bump)]
    pub nft_mint_authority: UncheckedAccount<'info>,

    /// The new NFT mint. Caller generates a fresh Keypair and passes it as a
    /// signer — Anchor creates the account with `init`. Decimals=0 and
    /// supply is enforced to stay at 1 by the master edition we create.
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = nft_mint_authority,
        mint::freeze_authority = nft_mint_authority,
        mint::token_program = token_program,
    )]
    pub mint: Account<'info, Mint>,

    /// Metaplex metadata PDA. Uninitialized at call time — created by the
    /// Metaplex CPI. Anchor can't derive it (different program owns it), so
    /// the caller computes it off-chain and we trust Metaplex to validate.
    ///
    /// CHECK: Validated by the Metaplex Token Metadata program.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// Metaplex master edition PDA. Same deal — created by Metaplex CPI.
    ///
    /// CHECK: Validated by the Metaplex Token Metadata program.
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,

    /// Destination token account. Created here if it doesn't exist.
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
        associated_token::token_program = token_program,
    )]
    pub recipient_ata: Account<'info, TokenAccount>,

    /// CHECK: Just a destination wallet — no discriminator needed.
    pub recipient: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    /// CHECK: Metaplex still checks rent sysvar on older instruction paths.
    pub rent: Sysvar<'info, Rent>,
}

/// Create a 1-of-1 NFT.
///
/// Steps:
///   1. Mint 1 token to `recipient_ata` (signed by `nft_mint_authority` PDA).
///   2. Create the Metaplex metadata account with name/symbol/uri.
///   3. Create the Metaplex master edition with `max_supply = Some(0)` — this
///      permanently sets the edition supply to 0 print editions and internally
///      transfers the mint_authority to the master edition PDA via a Token
///      program CPI. Since nobody holds the private key for the edition PDA,
///      further minting is impossible.
pub fn handler(
    ctx: Context<MintItemNft>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let bump = ctx.bumps.nft_mint_authority;
    let authority_seeds: &[&[u8]] = &[NFT_MINT_AUTHORITY_SEED, &[bump]];
    let signer_seeds: &[&[&[u8]]] = &[authority_seeds];

    // Step 1: Mint exactly 1 token to recipient.
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.nft_mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    // Step 2: Create Metaplex metadata account (name, symbol, URI).
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            mpl_token_metadata::ID,
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.nft_mint_authority.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.nft_mint_authority.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        ),
        mpl_token_metadata::types::DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        false, // is_mutable
        true,  // update_authority_is_signer
        None,  // collection_details
    )?;

    // Step 3: Create the master edition. max_supply = Some(0) means zero print
    // editions are allowed (this is the canonical 1-of-1 NFT). Metaplex
    // internally sets the mint_authority to the master edition PDA.
    create_master_edition_v3(
        CpiContext::new_with_signer(
            mpl_token_metadata::ID,
            CreateMasterEditionV3 {
                edition: ctx.accounts.master_edition.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.nft_mint_authority.to_account_info(),
                mint_authority: ctx.accounts.nft_mint_authority.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                metadata: ctx.accounts.metadata.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        ),
        Some(0), // max_supply = 0 print editions
    )?;

    msg!(
        "Minted NFT — mint: {}, recipient: {}",
        ctx.accounts.mint.key(),
        ctx.accounts.recipient.key()
    );
    Ok(())
}
