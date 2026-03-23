use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, program::invoke_signed, system_instruction};
use spl_token_2022::{extension::ExtensionType, instruction::initialize_mint2};
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;

pub mod errors;
pub mod instructions;
pub mod state;

pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("DdYgyunS5SdgqSA7j8Mr5C4vKaEK7MTEbMuWZqPGdC2B");

#[program]
pub mod magic_token {
    use super::*;

    /// Initialize the MagicToken mint (Token-2022 + MetadataPointer).
    pub fn initialize_magic_token(
        ctx: Context<InitializeMagicToken>,
        name: String,
        symbol: String,
        uri: String,
        marketplace_program: Pubkey,
    ) -> Result<()> {
        let mint_key = ctx.accounts.mint.key();
        let authority_key = ctx.accounts.mint_authority.key();

        let mint_space =
            ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&[
                ExtensionType::MetadataPointer,
            ])
            .map_err(|_| MagicError::SpaceCalculationFailed)?;

        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(mint_space);

        invoke(
            &system_instruction::create_account(
                &ctx.accounts.admin.key(),
                &mint_key,
                lamports,
                mint_space as u64,
                &spl_token_2022::ID,
            ),
            &[
                ctx.accounts.admin.to_account_info(),
                ctx.accounts.mint.to_account_info(),
            ],
        )?;

        invoke(
            &spl_token_2022::extension::metadata_pointer::instruction::initialize(
                &spl_token_2022::ID,
                &mint_key,
                Some(authority_key),
                Some(mint_key),
            )?,
            &[ctx.accounts.mint.to_account_info()],
        )?;

        invoke(
            &initialize_mint2(
                &spl_token_2022::ID,
                &mint_key,
                &authority_key,
                None,
                0,
            )?,
            &[ctx.accounts.mint.to_account_info()],
        )?;

        let token_metadata = TokenMetadata {
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            mint: mint_key,
            update_authority: Some(authority_key).try_into().unwrap(),
            additional_metadata: vec![],
        };
        let metadata_data_len = token_metadata.get_packed_len().unwrap_or(256);
        let new_total_space = mint_space + 12 + metadata_data_len;
        let extra_lamports = rent
            .minimum_balance(new_total_space)
            .saturating_sub(lamports);

        if extra_lamports > 0 {
            invoke(
                &system_instruction::transfer(
                    &ctx.accounts.admin.key(),
                    &mint_key,
                    extra_lamports,
                ),
                &[
                    ctx.accounts.admin.to_account_info(),
                    ctx.accounts.mint.to_account_info(),
                ],
            )?;
        }

        let auth_bump = ctx.bumps.mint_authority;
        let authority_seeds: &[&[u8]] = &[b"magic_mint_authority", &[auth_bump]];
        invoke_signed(
            &spl_token_metadata_interface::instruction::initialize(
                &spl_token_2022::ID,
                &mint_key,
                &authority_key,
                &mint_key,
                &authority_key,
                name,
                symbol,
                uri,
            ),
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
            ],
            &[authority_seeds],
        )?;

        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.mint = mint_key;
        config.marketplace_program = marketplace_program;
        config.bump = ctx.bumps.config;
        config.mint_authority_bump = ctx.bumps.mint_authority;
        Ok(())
    }

    /// Mint MagicToken to a recipient. CPI-gated: only callable by marketplace.
    pub fn mint_magic_token(
        ctx: Context<MintMagicToken>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, MagicError::InvalidAmount);

        let bump = ctx.accounts.config.mint_authority_bump;
        let seeds: &[&[u8]] = &[b"magic_mint_authority", &[bump]];

        anchor_spl::token_2022::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_2022::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.recipient_ata.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )
    }
}
