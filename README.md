# "Cossack Business" - Solana Version

## Introduction

This repository contains my implementation of the `task_Solana_2026` assignment. The project follows the original requirements:

- Rust programs built with Anchor
- six separate Solana programs
- Token-2022 resources
- Metaplex NFTs for crafted items
- TypeScript scripts for local test orchestration and devnet deployment
- end-to-end tests for the full game flow

The game itself is a small on-chain resource loop:

- players search for resources every 60 seconds
- resources are minted as Token-2022 tokens with `decimals = 0`
- players craft unique item NFTs from resource recipes
- items can be transferred to another player
- items can be sold to `marketplace` for `MagicToken`

## Code Requirements

| Parameter | Implementation in this repository |
|----------|-----------------------------------|
| Language | Rust for programs, TypeScript for scripts and tests |
| Framework | Anchor Framework `0.32.1` |
| Deployment network | Solana Devnet |
| Test coverage | Latest recorded program coverage summary: `355/355 (100.00%)` |
| Tooling | Anchor CLI, Solana CLI, Rust, Node.js |
| Scripts | TypeScript with `@coral-xyz/anchor` and `@solana/web3.js` |
| Documentation | Rust doc comments in program code and this README |
| README contents | Program IDs, architecture, run instructions, deploy instructions, examples |

## Versions Used

These versions were chosen deliberately and tested together as one stable stack:

- `anchor-cli 0.32.1`
- `anchor-lang 0.32.1`
- `anchor-spl 0.32.1`
- `solana-cli 2.3.0`
- `rustc 1.94.1`
- `node 20.x`

The point was not to use the newest possible versions, but to keep a stack that reliably builds, runs local tests, handles Token-2022 flows, and works with Metaplex NFT CPI calls.

## Program IDs

| Program | Program ID |
|--------|------------|
| `resource_manager` | `BnswUmgoVYBc4kkVbGethzDsAoRE4bGX3p19BJ4RuU43` |
| `item_nft` | `6ZFgUpi36moUoWHokvurbZfBY7wuG4tf28WkJR3d6EZP` |
| `crafting` | `EZdAg3bGtT4FwK9xcpUKM6UuJzYB8BMvXyKoHz3mS986` |
| `search` | `7yPJgKSZYcUCPgrEBmcQ7z86Frz57H6bsU5hBycStgp9` |
| `marketplace` | `E1nMz6JbstqDK9cEFhx1g3XrAJK8J2d9kvGiZTdYVaK9` |
| `magic_token` | `D6TYLNDSrga9igvU5NwHwjgYtxyeTvLNPXGB9fF5p1PB` |

The workspace uses a dedicated assignment wallet configured in `Anchor.toml`:

- wallet path: `./.keys/assignment-deployer.json`
- wallet pubkey: `HgcF3iz7rp1Xgd4LJCuAKphTtHWmTWSnh9co7esoqALT`

The `.keys` directory is ignored by git, so a fresh clone must create this wallet locally or provide `ANCHOR_WALLET`.

## Assignment Overview

### Base Resources (SPL Token-2022)

The game uses six base resources implemented as Token-2022 mints with `MetadataPointer`.

| ID | Name | Symbol | Decimals |
|----|------|--------|----------|
| 0 | Wood | `WOOD` | `0` |
| 1 | Iron | `IRON` | `0` |
| 2 | Gold | `GOLD` | `0` |
| 3 | Leather | `LEATHER` | `0` |
| 4 | Stone | `STONE` | `0` |
| 5 | Diamond | `DIAMOND` | `0` |

Resources are whole units, so all mints use `decimals = 0`.

### Unique Items (Metaplex NFTs)

Crafted items are minted as Metaplex NFTs.

| Item | Recipe | Marketplace Reward |
|------|--------|--------------------|
| Shablya Kozaka | `3 Iron + 1 Wood + 1 Leather` | `25 MagicToken` |
| Posokh Starishiyny | `2 Wood + 1 Gold + 1 Diamond` | `45 MagicToken` |
| Bronya Kharakternyka | `4 Leather + 2 Iron + 1 Gold` | `60 MagicToken` |
| Boyovyi Braslet | `4 Iron + 2 Gold + 2 Diamond` | `90 MagicToken` |

Each crafted item creates:

- a unique mint
- Metaplex metadata
- a master edition
- on-chain `ItemMetadata` owned by the project program

## Security and Access Model

### Token-2022 Resource Control

- direct resource minting through the base token program is blocked
- direct resource burning through the base token program is blocked
- direct resource transfers that bypass program flow are blocked
- authority is enforced through PDA checks and guarded token accounts

`resource_manager` uses program-owned authority and freeze/thaw flow to ensure that resource mint, burn, and transfer only happen through approved instructions.

### NFT Burn Control

- direct item minting outside `crafting` is blocked
- direct NFT burn through the token program is blocked
- NFT burn is allowed only through the sale flow in `marketplace`

To keep this property even after player-to-player transfer, `item_nft` uses the Metaplex delegate freeze/thaw pattern.

### NFT Transfer Model

Items can be transferred between players, but the receiver first prepares their token account through `prepare_item_receive`.

That preparation step exists for one reason: after the item is transferred, direct burn still has to remain blocked. The actual `transfer_item` transaction is owner-driven and does not require the receiver to co-sign the transfer itself.

## MagicToken Logic

`MagicToken` is a Token-2022 mint managed by its own program.

- players do not mint `MagicToken` directly
- `magic_token` accepts mint requests only from `marketplace`
- `marketplace` mints the configured payout after successful item sale

This keeps the payout path explicit:

`player item sale -> marketplace -> CPI to magic_token -> MagicToken payout`

## Search and Crafting Logic

### Search Program

- each player has a `Player` PDA
- search is allowed once every 60 seconds
- each successful search mints exactly three resources
- search rewards are minted through CPI into `resource_manager`

### Crafting Program

To craft an item, the player must:

1. own the required resources
2. sign the crafting transaction

During crafting:

- required resources are burned through CPI
- a new NFT mint is created
- metadata and master edition are created
- the NFT is transferred to the player

## Marketplace Logic

`marketplace` is implemented as a sink market:

- the player sells the item to the marketplace
- the NFT is thawed and burned
- the player receives `MagicToken`

This matches the assignment requirement that item sale pays out `MagicToken` while keeping burn restricted to the marketplace flow.

## Program Architecture

### Required Programs

| Program | Responsibility |
|---------|----------------|
| `resource_manager` | Creates and controls Token-2022 resource mints, resource guards, and `GameConfig` |
| `item_nft` | Mints Metaplex NFTs, manages receive preparation, guarded transfer, burn, and `ItemMetadata` |
| `crafting` | Validates recipes, burns resources through CPI, and crafts items through CPI |
| `search` | Enforces the 60-second cooldown and rewards resources through CPI |
| `marketplace` | Sells items, burns NFTs, and pays out `MagicToken` |
| `magic_token` | Owns and mints the `MagicToken` reward mint |

### Core PDA Accounts

```rust
#[account]
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; 6],
    pub magic_token_mint: Pubkey,
    pub item_prices: [u64; 4],
    pub bump: u8,
}

#[account]
pub struct Player {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub bump: u8,
}

#[account]
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub mint_seed: [u8; 32],
    pub bump: u8,
}
```

## Repository Structure

```text
crates/game_common/         shared seeds, IDs, recipes, constants
programs/resource_manager/  Token-2022 resources and GameConfig
programs/item_nft/          Metaplex NFT mint, guarded transfer, burn
programs/crafting/          crafting logic and CPI flow
programs/search/            search cooldown and reward flow
programs/marketplace/       item sale and MagicToken payout
programs/magic_token/       Token-2022 MagicToken mint
scripts/run_localnet_tests.ts
scripts/deploy_devnet.ts
tests/cases/                scenario groups
tests/suites/game.ts        end-to-end suite entrypoint
tests/support/              shared test context and helpers
```

## Testing

### What the Tests Cover

The end-to-end test suite covers:

- game initialization
- creation of all six resource mints
- registration of `MagicToken`
- Token-2022 metadata checks
- resource mint guard checks
- resource burn guard checks
- item mint guard checks
- search cooldown enforcement
- search account validation
- resource collection across players
- crafting success path
- player-to-player NFT transfer
- direct NFT burn rejection
- marketplace sale flow
- `MagicToken` payout path

### Coverage

Latest recorded full program coverage summary:

```text
programs/crafting/src/lib.rs          21/21    100.00%
programs/item_nft/src/lib.rs          77/77    100.00%
programs/magic_token/src/lib.rs       40/40    100.00%
programs/marketplace/src/lib.rs       16/16    100.00%
programs/resource_manager/src/lib.rs  120/120  100.00%
programs/search/src/lib.rs            81/81    100.00%
OVERALL                               355/355  100.00%
```

### Current Local Test Status

Latest full local run:

- `npm run lint`
- `npm run build`
- `npm test`

Result:

- `11 passing`

## Running Tests Locally

### Prerequisites

Install the following first:

- `solana-cli 2.3.0`
- `anchor-cli 0.32.1`
- Rust toolchain compatible with `rustc 1.94.1`
- Node.js `20.x`
- npm

Local tests also require network access to `mainnet-beta`, because the local validator clones the upgradeable Metaplex Token Metadata program at runtime. No bundled Metaplex `.so` file is required in the repository.

### Wallet Setup

Create the local assignment wallet file expected by `Anchor.toml`:

```bash
mkdir -p .keys
solana-keygen new --no-bip39-passphrase -o ./.keys/assignment-deployer.json
```

If you prefer another wallet path, set `ANCHOR_WALLET` or update `Anchor.toml`.

### Install Dependencies

```bash
npm install
```

### Run the Test Suite

```bash
npm test
```

`npm test` runs the TypeScript launcher in `scripts/run_localnet_tests.ts`. That script:

- builds the programs
- starts `solana-test-validator`
- clones the upgradeable Metaplex Token Metadata program from `mainnet-beta`
- airdrops local SOL to the configured wallet
- runs `anchor test --skip-local-validator --skip-build`

If you want to run the same flow manually:

```bash
solana-test-validator \
  --ledger /tmp/solana-task-validator \
  --reset \
  --url mainnet-beta \
  --rpc-port 19199 \
  --faucet-port 19202 \
  --gossip-port 19203 \
  --clone-upgradeable-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s

anchor build
anchor test --skip-build --skip-local-validator --provider.cluster http://127.0.0.1:19199
```

## Devnet Deployment

Run:

```bash
npm run deploy:devnet
```

The TypeScript deploy script in `scripts/deploy_devnet.ts`:

- switches Solana CLI config to `devnet`
- reads the wallet from `Anchor.toml`
- tries to airdrop enough SOL for deployment
- runs `anchor build`
- runs `anchor deploy --provider.cluster devnet`

If the devnet faucet hits a rate limit, fund the wallet manually and rerun the same command.

The assignment wallet used by this workflow is:

- `HgcF3iz7rp1Xgd4LJCuAKphTtHWmTWSnh9co7esoqALT`

Manual devnet deployment is still possible:

```bash
solana config set --url devnet
solana airdrop 2 HgcF3iz7rp1Xgd4LJCuAKphTtHWmTWSnh9co7esoqALT
anchor build
anchor deploy --provider.cluster devnet
```
