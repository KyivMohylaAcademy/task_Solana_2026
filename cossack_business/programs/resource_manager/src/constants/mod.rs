use anchor_lang::prelude::*;

/// Program ID of the search program, authorized to call mint_resource.
pub const AUTHORIZED_SEARCH_PROGRAM: Pubkey = pubkey!("8idBXvmxQEwn8BCVe5W8nzJqktRsgubP1eFUJ6XQLuRc");

/// Program ID of the crafting program, authorized to call mint_resource and burn_resource.
pub const AUTHORIZED_CRAFTING_PROGRAM: Pubkey = pubkey!("YR3AszQR5gP98pMuzFb81Apb5KCsFi7U1gsSxfFeocF");
