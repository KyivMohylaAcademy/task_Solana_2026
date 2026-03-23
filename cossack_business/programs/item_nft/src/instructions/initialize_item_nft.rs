use anchor_lang::prelude::*;
use crate::state::ItemNftConfig;

/// Accounts for storing authorized caller program IDs.
#[derive(Accounts)]
pub struct InitializeItemNft<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + ItemNftConfig::INIT_SPACE,
        seeds = [b"item_nft_config"],
        bump,
    )]
    pub config: Account<'info, ItemNftConfig>,

    /// CHECK: PDA used as NFT mint/update authority
    #[account(seeds = [b"nft_authority"], bump)]
    pub nft_authority: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
