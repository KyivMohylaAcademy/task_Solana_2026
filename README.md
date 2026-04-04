# Kozatskyi Biznes (Solana / Anchor)

Solana game economy implementation (resources, crafting, marketplace, and magic token), split into six Anchor programs with CPI-based access control.

## Repo Structure

```text
programs/
  resource_manager/
  search/
  crafting/
  item_nft/
  marketplace/
  magic_token/
shared/                 # shared enums + GameConfig layout
tests/                  # split integration test suites
  init.spec.ts
  search.spec.ts
  crafting.spec.ts
  marketplace.spec.ts
  marketplace-purchase.spec.ts
  magic-token.spec.ts
  utils/
```

## Game Mechanics Implemented

### Resources (Token-2022, 6 mints, decimals 0)

- `WOOD`, `IRON`, `GOLD`, `LEATHER`, `STONE`, `DIAMOND`.
- Minting is allowed only through authorized CPI path (`search -> resource_manager`).
- Burning for recipes is allowed only through authorized CPI path (`crafting -> resource_manager`).

### Search

- Each player has a `Player` PDA (`["player", owner]`).
- `search_resources` enforces a 60-second cooldown.
- Every successful search mints exactly 3 random resource units.

### Crafting

Recipes:

- Saber: `3 IRON + 1 WOOD + 1 LEATHER`
- Staff: `2 WOOD + 1 GOLD + 1 DIAMOND`
- Armor: `4 LEATHER + 2 IRON + 1 GOLD`
- Bracelet: `4 IRON + 2 GOLD + 2 DIAMOND`

Flow:

1. Burn recipe resources via CPI to `resource_manager`.
2. Mint item NFT via CPI to `item_nft`.

### Marketplace + MagicToken

- Listing moves NFT to escrow ATA owned by marketplace authority PDA.
- Delist returns NFT to seller and closes listing.
- Purchase:
  1. burn MagicToken from buyer via CPI to `magic_token`,
  2. mint MagicToken to seller via CPI to `magic_token`,
  3. burn escrowed NFT in marketplace.
- Direct magic token mint without marketplace signer is rejected.

## Prerequisites

- Solana CLI installed (`solana --version`)
- Rust toolchain installed (`rustc --version`, `cargo --version`)
- Anchor CLI compatible with `@coral-xyz/anchor@0.32.1`
- Node.js + Yarn

## Build

```bash
anchor build
```

## Testing

```bash
anchor test
```

### What Tests Cover

- Initialization of `GameConfig` and `MagicTokenConfig`
- Search minting (exactly 3 resources) + cooldown enforcement
- All crafting recipes + invalid item-type rejection
- Listing and delisting lifecycle
- Purchase failure when buyer lacks MagicToken
- Direct mint protection in `magic_token`
- PDA/signer/access-control paths via negative tests

## Devnet Deployment

### 1) Configure cluster + fund wallet

```bash
solana config set --url devnet
```

### 2) Update keys

1. Generate new keys:
```bash
anchor keys sync --provider.cluster devnet
```

2. Set new keys in `Anchor.toml`

3. Set new resource_manager address in `programs/item_nft/src/lib.rs` and `shared/src/lib.rs`

### 2) Build and deploy

```bash
anchor build
anchor deploy --provider.cluster devnet
```