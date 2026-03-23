use anchor_lang::prelude::*;
use resource_manager::{self as rm, cpi::accounts::BurnResource};

pub mod constants;
pub mod errors;
pub mod instructions;

pub use constants::*;
pub use errors::*;
pub use instructions::*;

declare_id!("HDSf5ek7LVHnG5vyKHT9JkSxjF2S9hGT5zsXQutkUbkS");

#[program]
pub mod crafting {
    use super::*;

    /// Craft an item by burning resources and minting an NFT.
    ///
    /// Remaining accounts layout (only for resources with non-zero recipe requirements):
    ///   [resource_mint_0, resource_ata_0, resource_mint_1, resource_ata_1, ...]
    /// Followed by NFT accounts:
    ///   [nft_mint, player_nft_ata, metadata_account, master_edition,
    ///    metadata_program, item_metadata, item_nft_config, nft_authority, rent_sysvar]
    ///
    /// `resource_ids` lists which resource IDs are included, in the same order as the
    /// remaining accounts. Only non-zero recipe entries should be passed.
    pub fn craft_item<'info>(
        ctx: Context<'_, '_, 'info, 'info, CraftItem<'info>>,
        item_type: u8,
        resource_ids: Vec<u8>,
    ) -> Result<()> {
        require!((item_type as usize) < RECIPES.len(), CraftError::InvalidItemType);
        let recipe = RECIPES[item_type as usize];

        // Reject duplicate resource_ids to prevent double-burn attacks
        let mut seen = [false; RESOURCE_COUNT];
        for &id in resource_ids.iter() {
            require!((id as usize) < RESOURCE_COUNT, CraftError::InvalidItemType);
            require!(!seen[id as usize], CraftError::DuplicateResourceId);
            seen[id as usize] = true;
        }

        for (id, &required) in recipe.iter().enumerate() {
            if required > 0 {
                require!(
                    resource_ids.contains(&(id as u8)),
                    CraftError::MissingResource
                );
            }
        }

        let caller_bump = ctx.bumps.caller_authority;
        let caller_seeds: &[&[u8]] = &[b"caller_authority", &[caller_bump]];

        let remaining = ctx.remaining_accounts;
        let burn_pair_count = resource_ids.len();
        require!(
            remaining.len() >= burn_pair_count * 2 + 9,
            CraftError::InvalidRemainingAccounts
        );

        for (pair_idx, &resource_id) in resource_ids.iter().enumerate() {
            let required = recipe[resource_id as usize];
            if required == 0 {
                continue;
            }

            let mint_info = remaining[pair_idx * 2].to_account_info();
            let ata_info = remaining[pair_idx * 2 + 1].to_account_info();

            rm::cpi::burn_resource(
                CpiContext::new_with_signer(
                    ctx.accounts.resource_manager_program.to_account_info(),
                    BurnResource {
                        caller_authority: ctx.accounts.caller_authority.to_account_info(),
                        player: ctx.accounts.player.to_account_info(),
                        game_config: ctx.accounts.game_config.to_account_info(),
                        resource_mint: mint_info,
                        player_ata: ata_info,
                        token_program: ctx.accounts.token_2022_program.to_account_info(),
                    },
                    &[caller_seeds],
                ),
                resource_id,
                required as u64,
            )?;
        }

        let nft_offset = burn_pair_count * 2;
        let nft_mint = remaining[nft_offset].to_account_info();
        let player_nft_ata = remaining[nft_offset + 1].to_account_info();
        let metadata_account = remaining[nft_offset + 2].to_account_info();
        let master_edition = remaining[nft_offset + 3].to_account_info();
        let metadata_program = remaining[nft_offset + 4].to_account_info();
        let item_metadata = remaining[nft_offset + 5].to_account_info();
        let item_nft_config = remaining[nft_offset + 6].to_account_info();
        let nft_authority = remaining[nft_offset + 7].to_account_info();
        let rent_sysvar = remaining[nft_offset + 8].to_account_info();

        item_nft::cpi::create_item(
            CpiContext::new_with_signer(
                ctx.accounts.item_nft_program.to_account_info(),
                item_nft::cpi::accounts::CreateItem {
                    caller_authority: ctx.accounts.caller_authority.to_account_info(),
                    config: item_nft_config,
                    nft_authority,
                    player: ctx.accounts.player.to_account_info(),
                    payer: ctx.accounts.player.to_account_info(),
                    nft_mint,
                    player_nft_ata,
                    item_metadata,
                    metadata_account,
                    master_edition,
                    metadata_program,
                    token_program: ctx.accounts.token_program.to_account_info(),
                    associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: rent_sysvar,
                },
                &[caller_seeds],
            ),
            item_type,
        )?;

        Ok(())
    }
}
