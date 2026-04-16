use anchor_lang::prelude::*;

#[error_code]
pub enum ResourceManagerError {
    #[msg("Resource id must be in range 0..6")]
    InvalidResourceId,
    #[msg("This resource mint has already been initialised")]
    ResourceMintAlreadyInitialised,
    #[msg("Resource mint has not been initialised yet")]
    ResourceMintNotInitialised,
    #[msg("Search program has not been registered on the GameConfig")]
    SearchProgramNotRegistered,
}
