/// crafting — combines resource tokens into game-item NFTs.
///
/// Recipes
/// -------
/// | Item      | ID | Resources required                          |
/// |-----------|----|---------------------------------------------|
/// | Saber     |  0 | 3× Iron (1), 1× Wood (0), 1× Leather (3)   |
/// | Staff     |  1 | 2× Wood (0), 1× Gold (2), 1× Diamond (5)   |
/// | Armor     |  2 | 4× Leather (3), 2× Iron (1), 1× Gold (2)   |
/// | Bracelet  |  3 | 4× Iron (1), 2× Gold (2), 2× Diamond (5)   |
///
/// Flow
/// ----
/// 1. Verify the player has provided the correct resource token accounts.
/// 2. Burn required resources via CPI to `resource_manager`.
/// 3. Create item NFT via CPI to `item_nft`.
use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};
use resource_manager::{
    self,
    cpi::{accounts::BurnResources, burn_resources},
    program::ResourceManager,
    resource_type,
};
use item_nft::{
    self,
    cpi::{accounts::CreateItem, create_item},
    program::ItemNft,
};

declare_id!("Crafting111111111111111111111111111111111111");

// ─── Recipe definitions ───────────────────────────────────────────────────────

/// (resource_type, amount) ingredient.
pub type Ingredient = (u8, u64);

/// Returns the recipe for a given item type.
pub fn recipe(item_type: u8) -> Result<Vec<Ingredient>> {
    match item_type {
        0 => Ok(vec![ // Шабля козака
            (resource_type::IRON,    3),
            (resource_type::WOOD,    1),
            (resource_type::LEATHER, 1),
        ]),
        1 => Ok(vec![ // Посох старійшини
            (resource_type::WOOD,    2),
            (resource_type::GOLD,    1),
            (resource_type::DIAMOND, 1),
        ]),
        2 => Ok(vec![ // Броня характерника
            (resource_type::LEATHER, 4),
            (resource_type::IRON,    2),
            (resource_type::GOLD,    1),
        ]),
        3 => Ok(vec![ // Бойовий браслет
            (resource_type::IRON,    4),
            (resource_type::GOLD,    2),
            (resource_type::DIAMOND, 2),
        ]),
        _ => err!(CraftingError::InvalidItemType),
    }
}

// ─── Program ──────────────────────────────────────────────────────────────────

#[program]
pub mod crafting {
    use super::*;

    /// Crafts an item of `item_type`.
    ///
    /// `remaining_accounts` must contain pairs of (resource_mint, player_token_account)
    /// for each ingredient in the recipe, in the same order as `recipe(item_type)`.
    /// Additionally the last two accounts must be the item_mint and
    /// player_item_token_account for the new NFT.
    ///
    /// The crafting_authority PDA of this program must match the value stored
    /// in resource_manager's GameConfig and item_nft's ItemNftConfig.
    pub fn craft_item(ctx: Context<CraftItem>, item_type: u8) -> Result<()> {
        let ingredients = recipe(item_type)?;
        let expected_remaining = ingredients.len() * 2 + 2; // ingredients + item_mint + item_ta

        require!(
            ctx.remaining_accounts.len() == expected_remaining,
            CraftingError::WrongAccountCount
        );

        let crafting_auth_bump = ctx.bumps.crafting_authority;
        let seeds: &[&[u8]] = &[b"crafting_authority", &[crafting_auth_bump]];
        let signer_seeds = &[seeds];

        // ── 1. Burn each ingredient ───────────────────────────────────────────
        for (i, (rt, amount)) in ingredients.iter().enumerate() {
            let mint_info = &ctx.remaining_accounts[i * 2];
            let ta_info   = &ctx.remaining_accounts[i * 2 + 1];

            burn_resources(
                CpiContext::new_with_signer(
                    ctx.accounts.resource_manager_program.to_account_info(),
                    BurnResources {
                        game_config:          ctx.accounts.game_config.to_account_info(),
                        authority:            ctx.accounts.crafting_authority.to_account_info(),
                        player:               ctx.accounts.player.to_account_info(),
                        resource_mint:        mint_info.clone(),
                        player_token_account: ta_info.clone(),
                        token_program:        ctx.accounts.token_program.to_account_info(),
                    },
                    signer_seeds,
                ),
                *rt,
                *amount,
            )?;
        }

        // ── 2. Create item NFT ────────────────────────────────────────────────
        let n = ingredients.len();
        let item_mint_info = &ctx.remaining_accounts[n * 2];
        let item_ta_info   = &ctx.remaining_accounts[n * 2 + 1];

        // Derive the item_metadata PDA seeds (done by item_nft program)
        create_item(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                CreateItem {
                    item_nft_config:      ctx.accounts.item_nft_config.to_account_info(),
                    item_nft_authority:   ctx.accounts.item_nft_authority.to_account_info(),
                    crafting_authority:   ctx.accounts.crafting_authority.to_account_info(),
                    player:               ctx.accounts.player.to_account_info(),
                    item_mint:            item_mint_info.clone(),
                    player_token_account: item_ta_info.clone(),
                    item_metadata:        ctx.accounts.item_metadata.to_account_info(),
                    payer:                ctx.accounts.player.to_account_info(),
                    token_program:        ctx.accounts.token_program.to_account_info(),
                    system_program:       ctx.accounts.system_program.to_account_info(),
                },
                signer_seeds,
            ),
            item_type,
        )?;

        emit!(ItemCrafted {
            player:    ctx.accounts.player.key(),
            item_type,
            item_mint: item_mint_info.key(),
        });
        Ok(())
    }
}

// ─── Accounts ─────────────────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(item_type: u8)]
pub struct CraftItem<'info> {
    /// This program's cross-program authority PDA.
    /// CHECK: derived via seeds
    #[account(seeds = [b"crafting_authority"], bump)]
    pub crafting_authority: UncheckedAccount<'info>,

    /// resource_manager GameConfig.
    /// CHECK: validated inside resource_manager CPI
    pub game_config: UncheckedAccount<'info>,

    /// item_nft program's ItemNftConfig.
    /// CHECK: validated inside item_nft CPI
    pub item_nft_config: UncheckedAccount<'info>,

    /// item_nft program's authority PDA.
    /// CHECK: validated inside item_nft CPI
    pub item_nft_authority: UncheckedAccount<'info>,

    /// The player crafting the item — must sign (owner check for burn).
    #[account(mut)]
    pub player: Signer<'info>,

    /// ItemMetadata PDA — will be initialised by item_nft.
    /// CHECK: Initialised via CPI to item_nft
    #[account(mut)]
    pub item_metadata: UncheckedAccount<'info>,

    pub resource_manager_program: Program<'info, ResourceManager>,
    pub item_nft_program:         Program<'info, ItemNft>,
    pub token_program:            Program<'info, Token2022>,
    pub system_program:           Program<'info, System>,
}

// ─── Events ───────────────────────────────────────────────────────────────────

#[event]
pub struct ItemCrafted {
    pub player:    Pubkey,
    pub item_type: u8,
    pub item_mint: Pubkey,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[error_code]
pub enum CraftingError {
    #[msg("Invalid item type — must be 0-3")]
    InvalidItemType,
    #[msg("Wrong number of accounts in remaining_accounts for this recipe")]
    WrongAccountCount,
}
