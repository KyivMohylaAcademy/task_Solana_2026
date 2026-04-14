//! Shared error codes for all programs in the Козацький бізнес workspace.

use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    #[msg("Search cooldown has not elapsed yet")]
    SearchTooSoon,

    #[msg("Player has insufficient resources for crafting")]
    InsufficientResources,

    #[msg("Invalid item type (must be 0–3)")]
    InvalidItemType,

    #[msg("Invalid resource kind (must be 0–5)")]
    InvalidResourceKind,

    #[msg("Caller is not authorized to invoke this instruction")]
    UnauthorizedCaller,

    #[msg("NFT does not belong to the seller")]
    WrongOwner,

    #[msg("Item type argument does not match the NFT metadata")]
    ItemTypeMismatch,

    #[msg("Only the admin may call this instruction")]
    AdminOnly,

    #[msg("GameConfig already initialized")]
    AlreadyInitialized,

    #[msg("MagicToken mint already set in GameConfig")]
    MagicMintAlreadySet,

    #[msg("Resource mint for this kind already created")]
    MintAlreadyCreated,

    #[msg("Arithmetic overflow")]
    Overflow,
}
