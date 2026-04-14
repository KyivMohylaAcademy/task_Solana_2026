# Setup

## Prerequisites

Install the required toolchain:

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Solana CLI 2.1.0
sh -c "$(curl -sSfL https://release.anza.xyz/v2.1.0/install)"

# Anchor 0.31.1 (via avm)
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install 0.31.1
avm use 0.31.1

# pnpm
npm install -g pnpm
```

## Install JS dependencies

```bash
pnpm install
```

## Generate a local wallet (if you don't have one)

```bash
solana-keygen new --outfile ~/.config/solana/id.json
```

## Build all programs

```bash
anchor build
```

## Run a local validator

```bash
solana-test-validator --reset
```

## Deploy to localnet

```bash
anchor deploy
```

## Run tests (bankrun — no local validator needed)

```bash
pnpm test
```

## Deploy to devnet

```bash
# Fund your wallet first
solana airdrop 2 --url devnet

anchor deploy --provider.cluster devnet
```

## Utility scripts

```bash
# Initialize programs on localnet
pnpm exec ts-node scripts/initialize.ts

# Register a player
pnpm exec ts-node scripts/register_player.ts

# Run the demo
pnpm exec ts-node scripts/demo.ts
```

## Program addresses

| Program          | Address                                        |
|------------------|------------------------------------------------|
| resource_manager | F28jgR2vTiCi8PN9FW5B3v7JcBsu2NEPTJiX4KGxx2mj |
| magic_token      | HfLuv435urC8rxobkUe89f2cEYAKFxPKuwQfuDAZzrzT |
| search           | 9ZEk766xrSnSqJ4ke1vY9FhiGJwXZk37YK1ApBQaB6Pg |
| item_nft         | FQ4ptApSkc8RjUW35BVqL8BeuMgMRSYGtzDEwy2GhERf |
| crafting         | B2mXTz3cVrn3UubqVTKyqyEWh6qTiVcCjn1DQw8azB65 |
| marketplace      | 8FCw32yjvmK8po3yjH3U6p4ZNSzm7H7iCWiwjR6JHkzx |
