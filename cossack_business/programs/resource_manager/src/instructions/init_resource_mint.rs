use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
    token_2022_extensions::token_metadata::{
        token_metadata_initialize, TokenMetadataInitialize,
    },
    token_interface::Mint,
    token_2022::Token2022,
};

/// Fixed overhead for a TokenMetadata extension entry — name/symbol/uri are added on top.
const TOKEN_METADATA_OVERHEAD: usize = 85;

#[derive(Accounts)]
#[instruction(resource_id: u8)]
pub struct InitResourceMint<'info> {
    /// Payer for account creation.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// New Token-2022 mint. Caller generates a fresh Keypair and passes it as signer.
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = resource_mint_auth,
        mint::freeze_authority = resource_mint_auth,
        extensions::metadata_pointer::authority = resource_mint_auth,
        extensions::metadata_pointer::metadata_address = mint,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: PDA that acts as mint authority for all resource mints.
    #[account(seeds = [b"resource_mint_auth"], bump)]
    pub resource_mint_auth: AccountInfo<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

/// Initialize a single Token-2022 resource mint with embedded metadata.
/// Must be called once per resource (6 times total).
pub fn handler(
    ctx: Context<InitResourceMint>,
    _resource_id: u8,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let bump = ctx.bumps.resource_mint_auth;
    let signer_seeds: &[&[&[u8]]] = &[&[b"resource_mint_auth", &[bump]]];

    // `token_metadata_initialize` reallocates the mint account but does not transfer lamports,
    // so we must top up the mint first.
    let metadata_len = TOKEN_METADATA_OVERHEAD + name.len() + symbol.len() + uri.len();
    let rent = Rent::get()?;
    let current_lamports = ctx.accounts.mint.to_account_info().lamports();
    let current_data_len = ctx.accounts.mint.to_account_info().data_len();
    let new_data_len = current_data_len + metadata_len;
    let required_lamports = rent.minimum_balance(new_data_len);

    if required_lamports > current_lamports {
        let extra = required_lamports - current_lamports;
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: ctx.accounts.mint.to_account_info(),
                },
            ),
            extra,
        )?;
    }

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        TokenMetadataInitialize {
            token_program_id: ctx.accounts.token_program.to_account_info(),
            metadata: ctx.accounts.mint.to_account_info(),
            update_authority: ctx.accounts.resource_mint_auth.to_account_info(),
            mint_authority: ctx.accounts.resource_mint_auth.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        },
        signer_seeds,
    );
    token_metadata_initialize(cpi_ctx, name, symbol, uri)?;

    Ok(())
}
