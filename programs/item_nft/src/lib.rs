use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    self, create_master_edition_v3, create_metadata_accounts_v3, freeze_delegated_account,
    mpl_token_metadata::accounts::{MasterEdition, Metadata},
    mpl_token_metadata::instructions::BurnNftCpiBuilder,
    mpl_token_metadata::types::DataV2,
    thaw_delegated_account, CreateMasterEditionV3, CreateMetadataAccountsV3,
    FreezeDelegatedAccount, ThawDelegatedAccount,
};
use anchor_spl::token::{self, Approve, Mint, MintTo, Token, TokenAccount, Transfer};
use game_common::{
    crafting_id, item_name, item_symbol, marketplace_id, recipe_for, CRAFTING_AUTHORITY_SEED,
    GAME_CONFIG_SEED, ITEM_AUTHORITY_SEED, ITEM_METADATA_SEED, ITEM_MINT_SEED,
    MARKETPLACE_AUTHORITY_SEED,
};

declare_id!("6ZFgUpi36moUoWHokvurbZfBY7wuG4tf28WkJR3d6EZP");

#[program]
pub mod item_nft {
    use super::*;

    /// Mints a Metaplex NFT for a crafted item. Only the crafting program may call it.
    pub fn mint_item(
        ctx: Context<MintItem>,
        item_type: u8,
        mint_seed: [u8; 32],
        uri: String,
    ) -> Result<()> {
        require!(recipe_for(item_type).is_some(), ErrorCode::InvalidItemType);
        validate_crafting_authority(
            ctx.accounts.crafting_authority.key(),
            ctx.accounts.game_config.key(),
        )?;
        validate_metadata_accounts(
            ctx.accounts.item_mint.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.master_edition.key(),
        )?;
        let game_config_key = ctx.accounts.game_config.key();

        let item_authority_seeds: &[&[u8]] = &[
            ITEM_AUTHORITY_SEED,
            game_config_key.as_ref(),
            &[ctx.bumps.item_authority],
        ];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.item_mint.to_account_info(),
                    to: ctx.accounts.player_item_account.to_account_info(),
                    authority: ctx.accounts.item_authority.to_account_info(),
                },
                &[item_authority_seeds],
            ),
            1,
        )?;

        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint: ctx.accounts.item_mint.to_account_info(),
                    mint_authority: ctx.accounts.item_authority.to_account_info(),
                    payer: ctx.accounts.player.to_account_info(),
                    update_authority: ctx.accounts.item_authority.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[item_authority_seeds],
            ),
            DataV2 {
                name: item_name(item_type)
                    .ok_or_else(|| error!(ErrorCode::InvalidItemType))?
                    .to_string(),
                symbol: item_symbol(item_type)
                    .ok_or_else(|| error!(ErrorCode::InvalidItemType))?
                    .to_string(),
                uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            true,
            true,
            None,
        )?;

        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    edition: ctx.accounts.master_edition.to_account_info(),
                    mint: ctx.accounts.item_mint.to_account_info(),
                    update_authority: ctx.accounts.item_authority.to_account_info(),
                    mint_authority: ctx.accounts.item_authority.to_account_info(),
                    payer: ctx.accounts.player.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[item_authority_seeds],
            ),
            Some(0),
        )?;

        token::approve(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Approve {
                    to: ctx.accounts.player_item_account.to_account_info(),
                    delegate: ctx.accounts.item_authority.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            1,
        )?;

        freeze_delegated_account(CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
            FreezeDelegatedAccount {
                metadata: ctx.accounts.metadata.to_account_info(),
                delegate: ctx.accounts.item_authority.to_account_info(),
                token_account: ctx.accounts.player_item_account.to_account_info(),
                edition: ctx.accounts.master_edition.to_account_info(),
                mint: ctx.accounts.item_mint.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
            &[item_authority_seeds],
        ))?;

        let item_metadata = &mut ctx.accounts.item_metadata;
        item_metadata.item_type = item_type;
        item_metadata.owner = ctx.accounts.player.key();
        item_metadata.mint = ctx.accounts.item_mint.key();
        item_metadata.mint_seed = mint_seed;
        item_metadata.bump = ctx.bumps.item_metadata;

        Ok(())
    }

    /// Creates or reuses the recipient ATA and pre-approves the freeze delegate for a future transfer.
    pub fn prepare_item_receive(ctx: Context<PrepareItemReceive>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.recipient_item_account.owner,
            ctx.accounts.recipient.key(),
            ErrorCode::InvalidRecipient
        );
        require_keys_eq!(
            ctx.accounts.recipient_item_account.mint,
            ctx.accounts.item_mint.key(),
            ErrorCode::InvalidMint
        );

        token::approve(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Approve {
                    to: ctx.accounts.recipient_item_account.to_account_info(),
                    delegate: ctx.accounts.item_authority.to_account_info(),
                    authority: ctx.accounts.recipient.to_account_info(),
                },
            ),
            1,
        )?;

        Ok(())
    }

    /// Transfers a crafted NFT between players while preserving the delegated freeze guard.
    pub fn transfer_item(ctx: Context<TransferItem>) -> Result<()> {
        validate_metadata_accounts(
            ctx.accounts.item_mint.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.master_edition.key(),
        )?;
        require!(
            ctx.accounts.owner.key() != ctx.accounts.recipient.key(),
            ErrorCode::SelfTransferNotAllowed
        );
        require_keys_eq!(
            ctx.accounts.item_metadata.owner,
            ctx.accounts.owner.key(),
            ErrorCode::InvalidOwner
        );
        require_keys_eq!(
            ctx.accounts.owner_item_account.owner,
            ctx.accounts.owner.key(),
            ErrorCode::InvalidOwner
        );
        require_keys_eq!(
            ctx.accounts.owner_item_account.mint,
            ctx.accounts.item_mint.key(),
            ErrorCode::InvalidMint
        );
        require_keys_eq!(
            ctx.accounts.recipient_item_account.owner,
            ctx.accounts.recipient.key(),
            ErrorCode::InvalidRecipient
        );
        require_keys_eq!(
            ctx.accounts.recipient_item_account.mint,
            ctx.accounts.item_mint.key(),
            ErrorCode::InvalidMint
        );
        require_keys_eq!(
            ctx.accounts.recipient_item_account.delegate.unwrap_or_default(),
            ctx.accounts.item_authority.key(),
            ErrorCode::RecipientAccountNotPrepared
        );
        require!(
            ctx.accounts.owner_item_account.amount == 1,
            ErrorCode::InvalidTokenAmount
        );
        let game_config_key = ctx.accounts.game_config.key();
        let item_authority_seeds: &[&[u8]] = &[
            ITEM_AUTHORITY_SEED,
            game_config_key.as_ref(),
            &[ctx.bumps.item_authority],
        ];

        let metadata_program = ctx.accounts.metadata_program.to_account_info();
        thaw_delegated_account(CpiContext::new_with_signer(
            metadata_program,
            ThawDelegatedAccount {
                metadata: ctx.accounts.metadata.to_account_info(),
                delegate: ctx.accounts.item_authority.to_account_info(),
                token_account: ctx.accounts.owner_item_account.to_account_info(),
                edition: ctx.accounts.master_edition.to_account_info(),
                mint: ctx.accounts.item_mint.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
            &[item_authority_seeds],
        ))?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.owner_item_account.to_account_info(),
                    to: ctx.accounts.recipient_item_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            1,
        )?;

        freeze_delegated_account(CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
            FreezeDelegatedAccount {
                metadata: ctx.accounts.metadata.to_account_info(),
                delegate: ctx.accounts.item_authority.to_account_info(),
                token_account: ctx.accounts.recipient_item_account.to_account_info(),
                edition: ctx.accounts.master_edition.to_account_info(),
                mint: ctx.accounts.item_mint.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
            &[item_authority_seeds],
        ))?;

        ctx.accounts.item_metadata.owner = ctx.accounts.recipient.key();
        Ok(())
    }

    /// Burns a crafted item. Only the marketplace program may trigger this path.
    pub fn burn_item(ctx: Context<BurnItem>) -> Result<()> {
        validate_marketplace_authority(ctx.accounts.marketplace_authority.key(), ctx.accounts.game_config.key())?;
        validate_metadata_accounts(
            ctx.accounts.item_mint.key(),
            ctx.accounts.metadata.key(),
            ctx.accounts.master_edition.key(),
        )?;
        require_keys_eq!(ctx.accounts.owner_item_account.owner, ctx.accounts.owner.key(), ErrorCode::InvalidOwner);
        require_keys_eq!(
            ctx.accounts.item_metadata.owner,
            ctx.accounts.owner.key(),
            ErrorCode::InvalidOwner
        );
        require_keys_eq!(ctx.accounts.owner_item_account.mint, ctx.accounts.item_mint.key(), ErrorCode::InvalidMint);
        require!(ctx.accounts.owner_item_account.amount == 1, ErrorCode::InvalidTokenAmount);
        let game_config_key = ctx.accounts.game_config.key();
        let item_authority_seeds: &[&[u8]] = &[
            ITEM_AUTHORITY_SEED,
            game_config_key.as_ref(),
            &[ctx.bumps.item_authority],
        ];

        thaw_item_account(
            ctx.accounts.metadata_program.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.item_authority.to_account_info(),
            ctx.accounts.owner_item_account.to_account_info(),
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.item_mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            &[item_authority_seeds],
        )?;

        BurnNftCpiBuilder::new(&ctx.accounts.metadata_program.to_account_info())
            .metadata(&ctx.accounts.metadata.to_account_info())
            .owner(&ctx.accounts.owner.to_account_info())
            .mint(&ctx.accounts.item_mint.to_account_info())
            .token_account(&ctx.accounts.owner_item_account.to_account_info())
            .master_edition_account(&ctx.accounts.master_edition.to_account_info())
            .spl_token_program(&ctx.accounts.token_program.to_account_info())
            .invoke()
            .map_err(Into::into)
    }
}

#[derive(Accounts)]
#[instruction(item_type: u8, mint_seed: [u8; 32], uri: String)]
pub struct MintItem<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID
    )]
    pub game_config: Account<'info, resource_manager::GameConfig>,
    pub crafting_authority: Signer<'info>,
    /// CHECK: PDA signer that owns mint authority for all crafted NFTs.
    #[account(mut, seeds = [ITEM_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub item_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = player,
        seeds = [ITEM_MINT_SEED, player.key().as_ref(), mint_seed.as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = item_authority,
        mint::freeze_authority = item_authority
    )]
    pub item_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = player,
        associated_token::mint = item_mint,
        associated_token::authority = player
    )]
    pub player_item_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = player,
        seeds = [ITEM_METADATA_SEED, item_mint.key().as_ref()],
        bump,
        space = 8 + ItemMetadata::INIT_SPACE
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    /// CHECK: PDA address is verified against the Metaplex metadata derivation.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: PDA address is verified against the Metaplex master edition derivation.
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: This is the fixed Metaplex Token Metadata program id.
    #[account(address = metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BurnItem<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID
    )]
    pub game_config: Account<'info, resource_manager::GameConfig>,
    pub marketplace_authority: Signer<'info>,
    /// CHECK: PDA delegate that freezes NFT token accounts after mint and thaws them before burn.
    #[account(mut, seeds = [ITEM_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub item_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub item_mint: Account<'info, Mint>,
    #[account(mut)]
    pub owner_item_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        close = owner,
        seeds = [ITEM_METADATA_SEED, item_mint.key().as_ref()],
        bump = item_metadata.bump
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    /// CHECK: PDA address is verified against the Metaplex metadata derivation.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: PDA address is verified against the Metaplex master edition derivation.
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: This is the fixed Metaplex Token Metadata program id.
    #[account(address = metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PrepareItemReceive<'info> {
    #[account(mut)]
    pub recipient: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID
    )]
    pub game_config: Account<'info, resource_manager::GameConfig>,
    /// CHECK: PDA delegate that freezes NFT token accounts after mint and after transfer.
    #[account(mut, seeds = [ITEM_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub item_authority: UncheckedAccount<'info>,
    pub item_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = item_mint,
        associated_token::authority = recipient
    )]
    pub recipient_item_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferItem<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: The recipient wallet only needs to provide a public key for ATA derivation.
    pub recipient: UncheckedAccount<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        seeds::program = resource_manager::ID
    )]
    pub game_config: Account<'info, resource_manager::GameConfig>,
    /// CHECK: PDA delegate that freezes NFT token accounts after mint and after transfer.
    #[account(mut, seeds = [ITEM_AUTHORITY_SEED, game_config.key().as_ref()], bump)]
    pub item_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub item_mint: Account<'info, Mint>,
    #[account(mut)]
    pub owner_item_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub recipient_item_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [ITEM_METADATA_SEED, item_mint.key().as_ref()],
        bump = item_metadata.bump
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    /// CHECK: PDA address is verified against the Metaplex metadata derivation.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: PDA address is verified against the Metaplex master edition derivation.
    #[account(mut)]
    pub master_edition: UncheckedAccount<'info>,
    /// CHECK: This is the fixed Metaplex Token Metadata program id.
    #[account(address = metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// On-chain record that ties the NFT mint back to the crafted item type.
#[account]
#[derive(InitSpace)]
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub mint_seed: [u8; 32],
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unknown item type.")]
    InvalidItemType,
    #[msg("Only the crafting program may mint item NFTs.")]
    UnauthorizedCraftingCall,
    #[msg("Only the marketplace program may burn item NFTs.")]
    UnauthorizedMarketplaceCall,
    #[msg("Metadata PDA does not match the mint.")]
    InvalidMetadataPda,
    #[msg("Master edition PDA does not match the mint.")]
    InvalidMasterEditionPda,
    #[msg("The provided token account does not belong to the owner.")]
    InvalidOwner,
    #[msg("The destination token account does not belong to the provided recipient.")]
    InvalidRecipient,
    #[msg("The provided mint does not match the token account.")]
    InvalidMint,
    #[msg("NFT accounts must hold exactly one token.")]
    InvalidTokenAmount,
    #[msg("The recipient must differ from the current owner.")]
    SelfTransferNotAllowed,
    #[msg("The recipient ATA must be prepared by the recipient before transfer.")]
    RecipientAccountNotPrepared,
}

fn validate_crafting_authority(authority: Pubkey, game_config: Pubkey) -> Result<()> {
    let expected = Pubkey::find_program_address(
        &[CRAFTING_AUTHORITY_SEED, game_config.as_ref()],
        &crafting_id(),
    )
    .0;
    require_keys_eq!(expected, authority, ErrorCode::UnauthorizedCraftingCall);
    Ok(())
}

fn validate_marketplace_authority(authority: Pubkey, game_config: Pubkey) -> Result<()> {
    let expected = Pubkey::find_program_address(
        &[MARKETPLACE_AUTHORITY_SEED, game_config.as_ref()],
        &marketplace_id(),
    )
    .0;
    require_keys_eq!(expected, authority, ErrorCode::UnauthorizedMarketplaceCall);
    Ok(())
}

fn validate_metadata_accounts(
    mint: Pubkey,
    metadata: Pubkey,
    master_edition: Pubkey,
) -> Result<()> {
    let expected_metadata = Metadata::find_pda(&mint).0;
    let expected_master_edition = MasterEdition::find_pda(&mint).0;
    require_keys_eq!(expected_metadata, metadata, ErrorCode::InvalidMetadataPda);
    require_keys_eq!(
        expected_master_edition,
        master_edition,
        ErrorCode::InvalidMasterEditionPda
    );
    Ok(())
}

fn thaw_item_account<'info>(
    metadata_program: AccountInfo<'info>,
    metadata: AccountInfo<'info>,
    delegate: AccountInfo<'info>,
    token_account: AccountInfo<'info>,
    edition: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    thaw_delegated_account(CpiContext::new_with_signer(
        metadata_program,
        ThawDelegatedAccount {
            metadata,
            delegate,
            token_account,
            edition,
            mint,
            token_program,
        },
        signer_seeds,
    ))
}
