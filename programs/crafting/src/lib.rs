use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Burn, Token2022};

declare_id!("EfvmR78Gm6o8dwTpBDMicigDREQFfvPd7nmW8VknbqK3");

/// Рецепти: [resource_index, amount] для кожного предмета
/// 0=WOOD, 1=IRON, 2=GOLD, 3=LEATHER, 4=STONE, 5=DIAMOND
const RECIPES: [&[(u8, u64)]; 4] = [
    &[(1, 3), (0, 1), (3, 1)],         // Шабля: 3×IRON + 1×WOOD + 1×LEATHER
    &[(0, 2), (2, 1), (5, 1)],         // Посох: 2×WOOD + 1×GOLD + 1×DIAMOND
    &[(3, 4), (1, 2), (2, 1)],         // Броня: 4×LEATHER + 2×IRON + 1×GOLD
    &[(1, 4), (2, 2), (5, 2)],         // Браслет: 4×IRON + 2×GOLD + 2×DIAMOND
];

#[program]
pub mod crafting {
    use super::*;

    /// Крафт предмета: спалює ресурси і створює ItemMetadata PDA
    pub fn craft_item<'a>(ctx: Context<'a, CraftItem<'a>>, item_type: u8) -> Result<()> {
        require!(item_type < 4, CraftingError::InvalidItemType);

        let recipe = RECIPES[item_type as usize];

        // Спалюємо ресурси через remaining_accounts
        // remaining_accounts: [mint_0, ata_0, mint_1, ata_1, ...]
        for (i, &(resource_idx, amount)) in recipe.iter().enumerate() {
            let mint_account = &ctx.remaining_accounts[i * 2];
            let player_ata = &ctx.remaining_accounts[i * 2 + 1];

            token_2022::burn(
                CpiContext::new(
                    anchor_spl::token_2022::ID,
                    Burn {
                        mint: mint_account.to_account_info(),
                        from: player_ata.to_account_info(),
                        authority: ctx.accounts.player.to_account_info(),
                    },
                ),
                amount,
            )?;

            msg!("Burned {} of resource {}", amount, resource_idx);
        }

        // Записуємо дані предмета
        let item = &mut ctx.accounts.item_metadata;
        item.item_type = item_type;
        item.owner = ctx.accounts.player.key();
        item.mint = ctx.accounts.item_mint.key();
        item.bump = ctx.bumps.item_metadata;

        msg!("Crafted item type={} mint={}", item_type, ctx.accounts.item_mint.key());
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
pub struct CraftItem<'info> {
    #[account(
        init,
        payer = player,
        space = ItemMetadata::LEN,
        seeds = [b"item", item_mint.key().as_ref()],
        bump
    )]
    pub item_metadata: Account<'info, ItemMetadata>,

    /// CHECK: mint акаунт нового предмета (NFT)
    pub item_mint: UncheckedAccount<'info>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    // remaining_accounts: [mint, ata] пари для кожного ресурсу в рецепті
}

#[error_code]
pub enum CraftingError {
    #[msg("Невірний тип предмета (0-3)")]
    InvalidItemType,
    #[msg("Недостатньо ресурсів для крафту")]
    InsufficientResources,
}