//! Seed constants for all PDA derivations in the Козацький бізнес workspace.

/// GameConfig PDA – root configuration, owned by resource_manager program.
pub const GAME_CONFIG_SEED: &[u8] = b"config";

/// Per-kind resource mint PDAs.
pub const RESOURCE_MINT_SEED: &[u8] = b"resource_mint";

/// Mint authority PDA for all 6 resource mints.
pub const RESOURCE_AUTHORITY_SEED: &[u8] = b"resource_authority";

/// Authority PDA that search program signs with when calling resource_manager.
pub const SEARCH_AUTHORITY_SEED: &[u8] = b"search_authority";

/// Authority PDA that crafting program signs with when calling resource_manager / item_nft.
pub const CRAFTING_AUTHORITY_SEED: &[u8] = b"crafting_authority";

/// Authority PDA that marketplace program signs with when calling item_nft / magic_token.
pub const MARKETPLACE_AUTHORITY_SEED: &[u8] = b"marketplace_authority";

/// MagicToken mint PDA – owned by magic_token program.
pub const MAGIC_MINT_SEED: &[u8] = b"magic_mint";

/// Mint authority PDA for the MagicToken mint.
pub const MAGIC_AUTHORITY_SEED: &[u8] = b"magic_authority";

/// Player account PDA – keyed by owner pubkey, owned by search program.
pub const PLAYER_SEED: &[u8] = b"player";

/// ItemMetadata PDA – keyed by the mpl-core asset address, owned by item_nft program.
pub const ITEM_SEED: &[u8] = b"item";

/// mpl-core collection authority PDA – owned by item_nft program.
pub const ITEM_COLLECTION_AUTHORITY_SEED: &[u8] = b"item_collection_authority";

/// mpl-core collection asset key stored in item_nft program state.
pub const ITEM_COLLECTION_SEED: &[u8] = b"item_collection";
