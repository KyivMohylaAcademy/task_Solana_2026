use anchor_lang::prelude::*;

declare_id!("HDtdF8EjnBeRuVFVA3TUQFi3oM8qA8iGCcfrCJbRar1e");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::SearchResources;
pub use instructions::*;

#[program]
pub mod search {
    use super::*;

    /// Search for resources with 60-second timer
    pub fn search_resources(
        ctx: Context<SearchResources>,
    ) -> Result<()> {
        instructions::search::search_resources(ctx)
    }
}
