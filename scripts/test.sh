#!/bin/bash

# Test script for Cossack Business Game
# Runs all tests with proper logging

set -e

echo "Running Cossack Business Game Tests..."
echo ""

# Run tests with increased timeout
anchor test --skip-build

echo ""
echo "All tests passed."
echo ""
echo "Test Coverage Summary:"
echo "========================="
echo "resource_manager - initialization, minting, burning"
echo "magic_token - initialization, minting restrictions"
echo "item_nft - creation, transfer, burning"
echo "search - player init, search cooldown, multiple searches"
echo "crafting - all item types, recipe validation"
echo "marketplace - listing, buying, selling, price updates"
