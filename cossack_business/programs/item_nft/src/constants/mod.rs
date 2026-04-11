use anchor_lang::prelude::*;

// NFT metadata URIs by item_type index.
// Replace REPLACE_ME with real hosted URLs before devnet deploy.
pub const NFT_NAMES: [&str; 4] = [
    "Cossack Saber",
    "Elder's Staff",
    "Kharakternyk's Armor",
    "Battle Bracelet",
];

pub const NFT_SYMBOLS: [&str; 4] = ["CSAB", "ESTF", "KARM", "BRCL"];

pub const NFT_URIS: [&str; 4] = [
    "https://REPLACE_ME/cossack-saber.json",
    "https://REPLACE_ME/elders-staff.json",
    "https://REPLACE_ME/kharakt-armor.json",
    "https://REPLACE_ME/battle-bracelet.json",
];

// Only the crafting program may call mint_nft.
pub const AUTHORIZED_CRAFTING_PROGRAM: Pubkey = pubkey!("YR3AszQR5gP98pMuzFb81Apb5KCsFi7U1gsSxfFeocF");

// Only the marketplace program may call burn_nft.
pub const AUTHORIZED_MARKETPLACE_PROGRAM: Pubkey = pubkey!("6mYp9XMhdaqcRq9xh4EDBmRDGaDEEphzEJzpPF5KEpvX");
