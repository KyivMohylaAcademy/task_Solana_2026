#!/bin/bash

# Build script for Cossack Business Game
# Builds all Solana programs

set -e

echo "Building Cossack Business Game Programs..."

# Clean previous builds
echo "Cleaning previous builds..."
anchor clean

# Build all programs
echo "Building programs..."
anchor build

# Display program IDs
echo ""
echo "Program Keypair Addresses:"
echo "================================"

for program in resource_manager magic_token item_nft search crafting marketplace; do
  address=$(solana address -k target/deploy/${program}-keypair.json 2>/dev/null || echo "Not found")
  echo "$program: $address"
done

echo ""
echo "Build complete."
echo ""
echo "Next steps:"
echo "1. Update declare_id! macros in each program's lib.rs"
echo "2. Update [programs.devnet] section in Anchor.toml"
echo "3. Run: anchor deploy"
