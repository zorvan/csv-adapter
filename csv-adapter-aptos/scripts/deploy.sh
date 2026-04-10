#!/usr/bin/env bash
# Deploy CSV Seal contracts on Aptos Testnet
# Usage: ./deploy.sh [network] [aptos-cli-path]
#   network: testnet (default), devnet, mainnet
#   aptos-cli-path: path to aptos binary (default: aptos)

set -euo pipefail

NETWORK="${1:-testnet}"
APTOS="${2:-aptos}"

echo "=== Aptos ${NETWORK} Deployment ==="
echo ""

# Check dependencies
if ! command -v "$APTOS" &>/dev/null; then
    echo "ERROR: aptos CLI not found. Install with: cargo install --git https://github.com/aptos-labs/aptos-core.git aptos"
    exit 1
fi

cd "$(dirname "$0")/.."

# Check profile exists
echo "Checking Aptos CLI profile..."
"$APTOS" config show-profiles 2>/dev/null || {
    echo "No profile found. Run: aptos init --network ${NETWORK}"
    exit 1
}

echo ""

# Build the package
echo "Building Move package..."
"$APTOS" move compile --package-dir contracts 2>&1 | tail -5
echo ""

# Publish to testnet
echo "Publishing to ${NETWORK}..."
PUBLISH_OUTPUT=$("$APTOS" move publish \
    --package-dir contracts \
    --profile default \
    --assume-yes \
    --json 2>&1)

echo "$PUBLISH_OUTPUT" > "scripts/deploy-output-${NETWORK}.json"

# Extract package ID from output
PACKAGE_ID=$(echo "$PUBLISH_OUTPUT" | python3 -c "
import sys, json
data = json.load(sys.stdin)
# Aptos publish output contains the transaction hash
# The package ID is derived from the publisher address
tx_hash = data.get('hash', '')
print(tx_hash)
" 2>/dev/null || echo "")

# Get the account address (this is the package ID for named addresses)
ACCOUNT=$("$APTOS" config show-profiles --profile default 2>/dev/null | grep "account" | head -1 | awk '{print $2}' || echo "")

echo ""
echo "=== DEPLOYMENT SUMMARY ==="
echo "Account: ${ACCOUNT}"
echo "Package: ${PACKAGE_ID}"
echo "Network: ${NETWORK}"
echo "Module: csv_seal::csv_seal"
echo "=========================="
echo ""
echo "Next steps:"
echo "1. The package is published under your account address: ${ACCOUNT}"
echo "2. Initialize LockRegistry:"
echo "   aptos move run --package-dir contracts --function-id ${ACCOUNT}::csv_seal::init_registry --profile default"
echo ""

# Initialize the LockRegistry
echo "Initializing LockRegistry..."
"$APTOS" move run \
    --function-id "${ACCOUNT}::csv_seal::init_registry" \
    --profile default \
    --assume-yes \
    --json 2>&1 | tee "scripts/registry-init-${NETWORK}.json" | tail -3

echo ""
echo "Deployment complete!"
