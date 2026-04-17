use anchor_lang::prelude::*;

declare_id!("HMCgFhEqKWroNqsDNo1RmMsyR7Wky2J7CtfDQf32WHKR");

/// Типи предметів
pub const ITEM_COSSACK_SWORD: u8 = 0;   // Шабля козака
pub const ITEM_ELDER_STAFF: u8 = 1;     // Посох старійшини
pub const ITEM_ARMOR: u8 = 2;           // Броня характерника
pub const ITEM_BRACELET: u8 = 3;        // Бойовий браслет

/// Рецепти крафту: (resource_index, amount)
pub const RECIPES: [[(u8, u8); 4]; 4] = [
    // Шабля козака: 3×IRON + 1×WOOD + 1×LEATHER + 0
    [(1, 3), (0, 1), (3, 1), (0, 0)],
    // Посох старійшини: 2×WOOD + 1×GOLD + 1×DIAMOND
    [(0, 2), (2, 1), (5, 1), (0, 0)],
    // Броня характерника: 4×LEATHER + 2×IRON + 1×GOLD
    [(3, 4), (1, 2), (2, 1), (0, 0)],
    // Бойовий браслет: 4×IRON + 2×GOLD + 2×DIAMOND
    [(1, 4), (2, 2), (5, 2), (0, 0)],
];

#[program]
pub mod item_nft {
    use super::*;

    /// Створює метадані предмета (NFT) — викликається через CPI з crafting
    pub fn create_item(ctx: Context<CreateItem>, item_type: u8) -> Result<()> {
        require!(item_type < 4, ItemError::InvalidItemType);

        let item = &mut ctx.accounts.item_metadata;
        item.item_type = item_type;
        item.owner = ctx.accounts.owner.key();
        item.mint = ctx.accounts.mint.key();
        item.bump = ctx.bumps.item_metadata;

        msg!("Item created: type={}, mint={}", item_type, ctx.accounts.mint.key());
        Ok(())
    }

    /// Передає предмет іншому гравцю
    pub fn transfer_item(ctx: Context<TransferItem>, new_owner: Pubkey) -> Result<()> {
        let item = &mut ctx.accounts.item_metadata;
        require!(
            item.owner == ctx.accounts.owner.key(),
            ItemError::NotOwner
        );
        item.owner = new_owner;
        msg!("Item transferred to: {}", new_owner);
        Ok(())
    }

    /// Спалює предмет — викликається тільки через CPI з marketplace
    pub fn burn_item(ctx: Context<BurnItem>) -> Result<()> {
        let item = &mut ctx.accounts.item_metadata;
        require!(
            item.owner == ctx.accounts.owner.key(),
            ItemError::NotOwner
        );
        item.owner = Pubkey::default();
        msg!("Item burned: mint={}", item.mint);
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
        payer = owner,
        space = ItemMetadata::LEN,
        seeds = [b"item", mint.key().as_ref()],
        bump
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// CHECK: mint акаунт предмета
    pub mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferItem<'info> {
    #[account(
        mut,
        seeds = [b"item", item_metadata.mint.as_ref()],
        bump = item_metadata.bump
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct BurnItem<'info> {
    #[account(
        mut,
        seeds = [b"item", item_metadata.mint.as_ref()],
        bump = item_metadata.bump
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    pub owner: Signer<'info>,
}

#[error_code]
pub enum ItemError {
    #[msg("Невірний тип предмета (0-3)")]
    InvalidItemType,
    #[msg("Ти не є власником цього предмета")]
    NotOwner,
}