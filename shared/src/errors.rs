//! Shared Anchor error codes surfaced by the game programs.

use anchor_lang::prelude::*;

/// Canonical game error codes reused across the full Solana program suite.
#[error_code]
pub enum GameErrorCode {
    /// Returned when a resource enum is derived from an out-of-range index.
    #[msg("Invalid resource type index.")]
    InvalidResourceTypeIndex,
    /// Returned when an item enum is derived from an out-of-range index.
    #[msg("Invalid item type index.")]
    InvalidItemTypeIndex,
    /// Returned when the static recipe array order no longer matches `ItemType`.
    #[msg("Recipe table is out of sync with item types.")]
    RecipeTableOutOfSync,
    /// Returned when a recipe entry would craft an item for zero resources.
    #[msg("Each recipe must require at least one resource.")]
    RecipeMustRequireAtLeastOneResource,
    /// Returned when an instruction requires the bootstrap admin signer.
    #[msg("Only the configured admin can execute this instruction.")]
    UnauthorizedAdmin,
    /// Returned when a provided resource mint does not match the canonical PDA.
    #[msg("The provided resource mint does not match the configured PDA.")]
    ResourceMintAddressMismatch,
    /// Returned when a provided reward mint does not match the configured mint.
    #[msg("The provided reward token mint does not match the configured mint.")]
    MagicTokenMintAddressMismatch,
    /// Returned when minting resources is attempted outside the search CPI path.
    #[msg("Only the search program may mint resources to players.")]
    UnauthorizedResourceMintCaller,
    /// Returned when burning resources is attempted outside the crafting CPI path.
    #[msg("Only the crafting program may burn resources from players.")]
    UnauthorizedResourceBurnCaller,
    /// Returned when reward minting is attempted outside the marketplace CPI path.
    #[msg("Only the marketplace program may mint reward tokens to players.")]
    UnauthorizedMagicMintCaller,
    /// Returned when item minting is attempted outside the crafting CPI path.
    #[msg("Only the crafting program may mint item NFTs.")]
    UnauthorizedItemMintCaller,
    /// Returned when item burning is attempted outside the marketplace CPI path.
    #[msg("Only the marketplace program may burn item NFTs.")]
    UnauthorizedItemBurnCaller,
    /// Returned when a player searches before the cooldown expires.
    #[msg("Search is still on cooldown for this player.")]
    SearchCooldownActive,
    /// Returned when a token mint or burn amount is zero.
    #[msg("Amount must be greater than zero.")]
    InvalidTokenAmount,
    /// Returned when a provided Metaplex metadata PDA does not match the mint.
    #[msg("The provided Metaplex metadata PDA does not match the mint.")]
    InvalidMetaplexMetadataAddress,
    /// Returned when a provided Metaplex master edition PDA does not match the mint.
    #[msg("The provided Metaplex master edition PDA does not match the mint.")]
    InvalidMetaplexMasterEditionAddress,
    /// Returned when an item token account is not the expected owner ATA.
    #[msg("The provided item token account does not match the expected owner ATA.")]
    InvalidItemTokenAccount,
    /// Returned when a resource token account is not the expected owner ATA.
    #[msg("The provided resource token account does not match the expected owner ATA.")]
    InvalidResourceTokenAccount,
    /// Returned when a reward-token account is not the expected owner ATA.
    #[msg("The provided reward token account does not match the expected owner ATA.")]
    InvalidMagicTokenAccount,
    /// Returned when an item metadata account does not belong to the given mint.
    #[msg("The provided item metadata account does not match the mint.")]
    ItemMetadataMintMismatch,
    /// Returned when an item metadata account encodes a different item type.
    #[msg("The provided item metadata account does not match the requested item type.")]
    ItemMetadataTypeMismatch,
    /// Returned when the player lacks enough resources for a recipe.
    #[msg("The player does not have enough resources for this recipe.")]
    InsufficientResourcesForRecipe,
    /// Returned when crafting receives an invalid remaining-accounts layout.
    #[msg("The provided crafting resource accounts do not match the recipe.")]
    InvalidCraftingResourceAccounts,
}
