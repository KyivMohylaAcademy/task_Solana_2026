#!/usr/bin/env bash
set -euo pipefail

# Deploy all programs to Solana Devnet and initialize game state.
#
# Prerequisites:
#   - Solana CLI configured with a funded devnet keypair
#   - Anchor CLI (v0.32+)
#   - Node.js / Yarn
#
# Usage:
#   chmod +x scripts/deploy-devnet.sh
#   ./scripts/deploy-devnet.sh

echo "=== Cossack Business — Devnet Deployment ==="

echo ""
echo "1. Switching to devnet..."
solana config set --url devnet

echo ""
echo "2. Checking wallet balance..."
BALANCE=$(solana balance | awk '{print $1}')
echo "   Balance: ${BALANCE} SOL"

echo ""
echo "3. Building programs..."
anchor build

PROGRAMS=(resource_manager magic_token item_nft search crafting marketplace)

echo ""
echo "4. Deploying programs..."
for p in "${PROGRAMS[@]}"; do
  echo "   Deploying ${p}..."
  anchor deploy --program-name "$p" --provider.cluster devnet || {
    echo "   ⚠ ${p} deploy failed (may already be deployed). Trying upgrade..."
    anchor upgrade --program-name "$p" --provider.cluster devnet \
      "target/deploy/${p}.so" || echo "   ⚠ ${p} upgrade also failed"
  }
done

echo ""
echo "5. Initializing game state..."
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
ANCHOR_WALLET=~/.config/solana/id.json \
  npx ts-node scripts/initialize.ts

echo ""
echo "=== Deployment complete ==="
echo ""
echo "Program IDs (from Anchor.toml):"
for p in "${PROGRAMS[@]}"; do
  ID=$(grep "^${p}" Anchor.toml | head -1 | awk -F'"' '{print $2}')
  echo "  ${p} = ${ID}"
done
