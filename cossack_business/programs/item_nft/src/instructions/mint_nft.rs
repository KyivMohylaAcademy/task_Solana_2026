use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{self, Mint, Token, TokenAccount}};
use mpl_token_metadata::{
    instructions::{
        CreateMasterEditionV3Builder, CreateMetadataAccountV3Builder,
    },
    types::{Creator, DataV2},
};
use solana_program::program::invoke_signed;
use crate::{
    constants::{AUTHORIZED_CRAFTING_PROGRAM, NFT_NAMES, NFT_SYMBOLS, NFT_URIS},
    errors::ItemNftError,
    state::ItemMetadata,
};

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The recipient wallet.
    /// CHECK: Signer who receives the NFT.
    #[account(mut)]
    pub recipient: Signer<'info>,

    /// CHECK: CPI authority PDA from the calling crafting program.
    /// Verified in handler against AUTHORIZED_CRAFTING_PROGRAM.
    pub cpi_auth: AccountInfo<'info>,

    /// New NFT mint. Client-generated Keypair, unique per NFT.
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = nft_authority,
        mint::freeze_authority = nft_authority,
    )]
    pub nft_mint: Box<Account<'info, Mint>>,

    /// Recipient's ATA for the NFT. Created inside the handler (after nft_mint init) if absent.
    /// CHECK: validated implicitly by the create CPI + token::mint_to + token::freeze_account.
    #[account(mut)]
    pub recipient_nft_ata: UncheckedAccount<'info>,

    /// CHECK: PDA with authority over NFT mints.
    #[account(seeds = [b"nft_authority"], bump)]
    pub nft_authority: AccountInfo<'info>,

    /// CHECK: Metaplex metadata account — derived by Metaplex program.
    #[account(mut)]
    pub metadata: AccountInfo<'info>,

    /// CHECK: Metaplex master edition account — derived by Metaplex program.
    #[account(mut)]
    pub master_edition: AccountInfo<'info>,

    /// CHECK: Metaplex Token Metadata program.
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,

    /// ItemMetadata PDA storing item_type and original owner.
    #[account(
        init,
        payer = payer,
        space = ItemMetadata::LEN,
        seeds = [b"item_metadata", nft_mint.key().as_ref()],
        bump,
    )]
    pub item_metadata: Box<Account<'info, ItemMetadata>>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Separate stack frame for ATA creation inside the handler.
/// The ATA is created here (not in try_accounts) so that the nft_mint already exists
/// when the ATA program validates it. try_accounts runs init on nft_mint first.
#[inline(never)]
fn create_nft_ata_if_needed<'info>(
    payer: AccountInfo<'info>,
    ata: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    associated_token_program: AccountInfo<'info>,
) -> Result<()> {
    if ata.data_is_empty() {
        anchor_spl::associated_token::create(CpiContext::new(
            associated_token_program,
            anchor_spl::associated_token::Create {
                payer,
                associated_token: ata,
                authority,
                mint,
                system_program,
                token_program,
            },
        ))?;
    }
    Ok(())
}

/// Separate stack frame for CreateMetadataAccountV3 invoke_signed.
/// Extracted to avoid "Access violation in stack frame" — the builder + DataV2
/// allocate enough stack that keeping everything in one function overflows the
/// 4096-byte SBF frame limit when called via CPI from crafting.
#[inline(never)]
fn create_metadata_cpi<'info>(
    metadata: AccountInfo<'info>,
    nft_mint: AccountInfo<'info>,
    nft_authority: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    token_metadata_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let create_metadata_ix = CreateMetadataAccountV3Builder::new()
        .metadata(metadata.key())
        .mint(nft_mint.key())
        .mint_authority(nft_authority.key())
        .payer(payer.key())
        .update_authority(nft_authority.key(), true)
        .data(DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: Some(vec![Creator {
                address: nft_authority.key(),
                verified: true,
                share: 100,
            }]),
            collection: None,
            uses: None,
        })
        .is_mutable(true)
        .instruction();

    invoke_signed(
        &create_metadata_ix,
        &[
            metadata,
            nft_mint,
            nft_authority.clone(), // mint_authority
            payer,
            nft_authority,         // update_authority
            system_program,
            token_metadata_program,
        ],
        signer_seeds,
    )?;
    Ok(())
}

/// Separate stack frame for CreateMasterEditionV3 invoke_signed.
#[inline(never)]
fn create_master_edition_cpi<'info>(
    master_edition: AccountInfo<'info>,
    nft_mint: AccountInfo<'info>,
    nft_authority: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    metadata: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    token_metadata_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let create_edition_ix = CreateMasterEditionV3Builder::new()
        .edition(master_edition.key())
        .mint(nft_mint.key())
        .update_authority(nft_authority.key())
        .mint_authority(nft_authority.key())
        .payer(payer.key())
        .metadata(metadata.key())
        .max_supply(0)
        .instruction();

    invoke_signed(
        &create_edition_ix,
        &[
            master_edition,
            nft_mint,
            nft_authority.clone(), // update_authority
            nft_authority,         // mint_authority
            payer,
            metadata,
            token_program,
            system_program,
            token_metadata_program,
        ],
        signer_seeds,
    )?;
    Ok(())
}

/// Mint one NFT of the given item_type to the recipient.
pub fn handler(ctx: Context<MintNft>, item_type: u8) -> Result<()> {
    require!(item_type < 4, ItemNftError::InvalidItemType);

    // Verify the caller is the authorized crafting program.
    let expected = Pubkey::find_program_address(&[b"cpi_auth"], &AUTHORIZED_CRAFTING_PROGRAM).0;
    require_keys_eq!(
        ctx.accounts.cpi_auth.key(),
        expected,
        ItemNftError::Unauthorized
    );

    let bump = ctx.bumps.nft_authority;
    let signer_seeds: &[&[&[u8]]] = &[&[b"nft_authority", &[bump]]];

    let name = NFT_NAMES[item_type as usize].to_string();
    let symbol = NFT_SYMBOLS[item_type as usize].to_string();
    let uri = NFT_URIS[item_type as usize].to_string();

    create_metadata_cpi(
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_authority.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        signer_seeds,
        name,
        symbol,
        uri,
    )?;

    // ATA creation runs after try_accounts has executed init on nft_mint,
    // so the mint is live on-chain when the ATA program validates it.
    create_nft_ata_if_needed(
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.recipient_nft_ata.to_account_info(),
        ctx.accounts.recipient.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.associated_token_program.to_account_info(),
    )?;

    // Must happen before CreateMasterEditionV3 — Metaplex requires supply == 1 at that point.
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.recipient_nft_ata.to_account_info(),
                authority: ctx.accounts.nft_authority.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    // Metaplex revokes the freeze authority when creating the master edition, so the ATA
    // cannot be frozen before this call (it could never be thawed). Burn protection is enforced
    // solely via the cpi_auth guard on burn_nft.
    create_master_edition_cpi(
        ctx.accounts.master_edition.to_account_info(),
        ctx.accounts.nft_mint.to_account_info(),
        ctx.accounts.nft_authority.to_account_info(),
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.metadata.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.token_metadata_program.to_account_info(),
        signer_seeds,
    )?;

    let meta = &mut ctx.accounts.item_metadata;
    meta.item_type = item_type;
    meta.owner = ctx.accounts.recipient.key();
    meta.mint = ctx.accounts.nft_mint.key();
    meta.bump = ctx.bumps.item_metadata;

    Ok(())
}
