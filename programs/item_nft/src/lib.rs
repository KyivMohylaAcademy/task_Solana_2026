use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
    Metadata,
};
use anchor_spl::token_interface::{self, Mint, MintTo, Burn, TokenAccount, TokenInterface};

declare_id!("3kSdBAJ6jmN6bdXR14u5LWTP98EcaSYbEi1AmZJbpTdp");

pub const ITEM_SHABLIA: u8 = 0;
pub const ITEM_POSOKH: u8 = 1;
pub const ITEM_BRONYA: u8 = 2;
pub const ITEM_BRASLET: u8 = 3;
pub const NUM_ITEMS: usize = 4;

pub const ITEM_NAMES: [&str; NUM_ITEMS] = [
    "Шабля козака",
    "Посох старійшини",
    "Броня характерника",
    "Бойовий браслет",
];

pub const ITEM_SYMBOLS: [&str; NUM_ITEMS] = ["SHABLIA", "POSOKH", "BRONYA", "BRASLET"];

pub const ITEM_URIS: [&str; NUM_ITEMS] = [
    "https://kozatsky.game/shablia.json",
    "https://kozatsky.game/posokh.json",
    "https://kozatsky.game/bronya.json",
    "https://kozatsky.game/braslet.json",
];

pub const ITEM_PRICES: [u64; NUM_ITEMS] = [100, 150, 200, 300];

#[program]
pub mod item_nft {
    use super::*;

    pub fn create_item(ctx: Context<CreateItem>, item_type: u8) -> Result<()> {
        require!((item_type as usize) < NUM_ITEMS, ItemError::InvalidItemType);

        let idx = item_type as usize;

        token_interface::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.item_mint.to_account_info(),
                    to: ctx.accounts.player_item_account.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            1,
        )?;

        create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    mint: ctx.accounts.item_mint.to_account_info(),
                    mint_authority: ctx.accounts.player.to_account_info(),
                    payer: ctx.accounts.player.to_account_info(),
                    update_authority: ctx.accounts.player.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ),
            DataV2 {
                name: ITEM_NAMES[idx].to_string(),
                symbol: ITEM_SYMBOLS[idx].to_string(),
                uri: ITEM_URIS[idx].to_string(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            true,
            true,
            None,
        )?;

        let item_metadata = &mut ctx.accounts.item_metadata;
        item_metadata.item_type = item_type;
        item_metadata.owner = ctx.accounts.player.key();
        item_metadata.mint = ctx.accounts.item_mint.key();
        item_metadata.bump = ctx.bumps.item_metadata;

        msg!(
            "ItemNFT: Created {} (mint: {})",
            ITEM_NAMES[idx],
            ctx.accounts.item_mint.key()
        );
        Ok(())
    }

    pub fn burn_item(ctx: Context<BurnItem>) -> Result<()> {
        token_interface::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.item_mint.to_account_info(),
                    from: ctx.accounts.player_item_account.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                },
            ),
            1,
        )?;

        let item_metadata = &ctx.accounts.item_metadata;
        msg!(
            "ItemNFT: Burned item type {} (mint: {})",
            item_metadata.item_type,
            item_metadata.mint
        );

        Ok(())
    }
}

#[account]
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

impl ItemMetadata {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 1;
}

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CreateItem<'info> {
    #[account(
        init,
        payer = player,
        space = ItemMetadata::LEN,
        seeds = [b"item_metadata", item_mint.key().as_ref()],
        bump,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    #[account(
        init,
        payer = player,
        mint::decimals = 0,
        mint::authority = player,
    )]
    pub item_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = player,
        associated_token::mint = item_mint,
        associated_token::authority = player,
    )]
    pub player_item_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: validated by Metaplex program
    #[account(mut)]
    pub metadata_account: AccountInfo<'info>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BurnItem<'info> {
    #[account(
        mut,
        seeds = [b"item_metadata", item_mint.key().as_ref()],
        bump = item_metadata.bump,
        close = player,
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    #[account(mut)]
    pub item_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = item_mint,
        associated_token::authority = player,
    )]
    pub player_item_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

#[error_code]
pub enum ItemError {
    #[msg("Invalid item type. Must be 0-3.")]
    InvalidItemType,
}
