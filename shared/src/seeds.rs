//! PDA seed constants and helper functions shared between on-chain and off-chain code.

use anchor_lang::prelude::*;

use crate::types::ResourceType;

/// Seed for each program's signer PDA used in CPI flows.
pub const PROGRAM_AUTHORITY_SEED: &[u8] = b"program_authority";
/// Seed for the global `GameConfig` account.
pub const GAME_CONFIG_SEED: &[u8] = b"game_config";
/// Seed for the per-wallet `Player` account in the search program.
pub const PLAYER_SEED: &[u8] = b"player";
/// Seed for the per-mint metadata account tracked by `item_nft`.
pub const ITEM_METADATA_SEED: &[u8] = b"item_metadata";
/// Seed prefix for every resource mint PDA.
pub const RESOURCE_MINT_SEED: &[u8] = b"resource_mint";
/// Seed for the canonical MagicToken mint PDA.
pub const MAGIC_TOKEN_MINT_SEED: &[u8] = b"magic_token_mint";

/// Program ID for the `resource_manager` program.
pub const RESOURCE_MANAGER_PROGRAM_ID: Pubkey =
    anchor_lang::pubkey!("CwwxNgkg1s8rjRAAN9zcvLgBCBhXTvCu4L1oAupBqiTe");
/// Program ID for the `item_nft` program.
pub const ITEM_NFT_PROGRAM_ID: Pubkey =
    anchor_lang::pubkey!("31YqF1ymwThcZTyGCmx6Uqnvjev15JRkWvMSJoxc3wve");
/// Program ID for the `crafting` program.
pub const CRAFTING_PROGRAM_ID: Pubkey =
    anchor_lang::pubkey!("A14WMVRTuuS4JtVcg22BuiWHvhJx1ZhxJS5CrWfy2tHh");
/// Program ID for the `search` program.
pub const SEARCH_PROGRAM_ID: Pubkey =
    anchor_lang::pubkey!("5vrMHniMhyCnZBK5PWTMMF2w886LDc1Kd3GdN17cbPGh");
/// Program ID for the `marketplace` program.
pub const MARKETPLACE_PROGRAM_ID: Pubkey =
    anchor_lang::pubkey!("3cPgZBSjpvcuD5FmhGQfCSBFXnz3ZMs573u8UDszgpeW");
/// Program ID for the `magic_token` program.
pub const MAGIC_TOKEN_PROGRAM_ID: Pubkey =
    anchor_lang::pubkey!("Bvw1CY1ZBu7jE2zmmKkWKe75LfoQvudwT11YxGYaLGW");
/// Program ID for the Metaplex Token Metadata program.
pub const TOKEN_METADATA_PROGRAM_ID: Pubkey =
    anchor_lang::pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

/// Derives the signer PDA for a given program.
pub fn find_program_authority_address(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PROGRAM_AUTHORITY_SEED], program_id)
}

/// Derives the global `GameConfig` PDA for a given program.
pub fn find_game_config_address(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[GAME_CONFIG_SEED], program_id)
}

/// Derives the search-player PDA for a wallet owner.
pub fn find_player_address(program_id: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PLAYER_SEED, owner.as_ref()], program_id)
}

/// Derives the `item_nft` metadata PDA for a minted NFT.
pub fn find_item_metadata_address(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[ITEM_METADATA_SEED, mint.as_ref()], program_id)
}

/// Derives the resource mint PDA for a specific resource type.
pub fn find_resource_mint_address(
    program_id: &Pubkey,
    resource_type: ResourceType,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[RESOURCE_MINT_SEED, &[resource_type as u8]], program_id)
}

/// Derives the canonical MagicToken mint PDA.
pub fn find_magic_token_mint_address(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MAGIC_TOKEN_MINT_SEED], program_id)
}
