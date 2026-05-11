#!/bin/bash
# Foundry-based deployment script for CSV contracts to Sepolia
#
# Prerequisites:
#   - foundry installed (https://getfoundry.sh/)
#   - SEPOLIA_RPC_URL set to Sepolia endpoint
#   - DEPLOYER_KEY set to deployer private key (with 0x prefix)
#
# Usage:
#   export SEPOLIA_RPC_URL="https://sepolia.infura.io/v3/YOUR_KEY"
#   export DEPLOYER_KEY="0x..."
#   ./deploy-foundry.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  CSV Protocol - Foundry Deployment${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check prerequisites
check_prereq() {
    if [ -z "$1" ]; then
        echo -e "${RED}Error: $2 is not set${NC}"
        echo -e "${YELLOW}Please set it with: export $2=...${NC}"
        exit 1
    fi
}

check_prereq "$SEPOLIA_RPC_URL" "SEPOLIA_RPC_URL"
check_prereq "$DEPLOYER_KEY" "DEPLOYER_KEY"

# Validate private key format
if [[ ! $DEPLOYER_KEY =~ ^0x[0-9a-fA-F]{64}$ ]]; then
    echo -e "${RED}Error: DEPLOYER_KEY must be 66 characters (0x + 64 hex)${NC}"
    echo -e "${YELLOW}Format: 0x1234...${NC}"
    exit 1
fi

echo -e "${BLUE}Configuration:${NC}"
echo -e "  RPC URL: ${SEPOLIA_RPC_URL:0:30}..."
echo -e "  Deployer: ${DEPLOYER_KEY:0:10}...${DEPLOYER_KEY: -8}"
echo ""

# Change to contracts directory
cd "$(dirname "$0")/../contracts"

# Check if foundry is available
if ! command -v forge &> /dev/null; then
    echo -e "${RED}Error: foundry not found. Install with: curl -L https://foundry.paradigm.xyz | bash${NC}"
    exit 1
fi

echo -e "${BLUE}Step 1: Deploy CSVLock Contract${NC}"
echo "----------------------------------------"
LOCK_OUTPUT=$(forge script script/Deploy.s.sol \
    --rpc-url "$SEPOLIA_RPC_URL" \
    --private-key "$DEPLOYER_KEY" \
    --broadcast \
    --verify \
    --json)

# Extract CSVLock address
CSVLOCK_ADDR=$(echo "$LOCK_OUTPUT" | jq -r '.transactions[] | select(.contractName == "CSVLock") | .contractAddress')
CSVLOCK_TX=$(echo "$LOCK_OUTPUT" | jq -r '.transactions[] | select(.contractName == "CSVLock") | .hash')

if [ -n "$CSVLOCK_ADDR" ] && [ "$CSVLOCK_ADDR" != "null" ]; then
    echo -e "${GREEN}✓ CSVLock deployed!${NC}"
    echo -e "  Address: ${CSVLOCK_ADDR}"
    echo -e "  TX Hash: ${CSVLOCK_TX}"
else
    echo -e "${RED}✗ CSVLock deployment failed${NC}"
    echo "$LOCK_OUTPUT"
    exit 1
fi

echo ""
echo -e "${BLUE}Step 2: Update Chain Configuration${NC}"
echo "----------------------------------------"

# Update the chains/ethereum.toml file
CHAIN_CONFIG_FILE="../../chains/ethereum.toml"

if [ -f "$CHAIN_CONFIG_FILE" ]; then
    # Backup original config
    cp "$CHAIN_CONFIG_FILE" "${CHAIN_CONFIG_FILE}.backup"
    
    # Update testnet section with deployed addresses
    if grep -q "\[testnet\]" "$CHAIN_CONFIG_FILE"; then
        # Update existing testnet section
        sed -i.bak "/\[testnet\]/,/^\[/ s/^lock_contract_address.*/lock_contract_address = \"${CSVLOCK_ADDR}\"/" "$CHAIN_CONFIG_FILE"
    else
        # Add testnet section
        cat >> "$CHAIN_CONFIG_FILE" << EOF

[testnet]
network = "sepolia"
chain_id = 11155111
rpc_url = "\$SEPOLIA_RPC_URL"
lock_contract_address = "${CSVLOCK_ADDR}"
mint_contract_address = ""
finality_depth = 12
use_checkpoint_finality = false
EOF
    fi
    
    echo -e "${GREEN}✓ Updated ${CHAIN_CONFIG_FILE}${NC}"
    echo -e "  Added lock_contract_address: ${CSVLOCK_ADDR}"
else
    echo -e "${YELLOW}Warning: ${CHAIN_CONFIG_FILE} not found${NC}"
    echo -e "  Please manually add lock_contract_address = \"${CSVLOCK_ADDR}\" to [testnet] section"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Deployment Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${BLUE}Deployed Contract:${NC}"
echo -e "  CSVLock: ${CSVLOCK_ADDR}"
echo ""
echo -e "${BLUE}View on Etherscan:${NC}"
echo -e "  https://sepolia.etherscan.io/address/${CSVLOCK_ADDR}"
echo ""

# Save deployment info
DEPLOY_FILE="../../deploy-output-ethereum-$(date +%Y%m%d-%H%M%S).json"
cat > "$DEPLOY_FILE" << EOF
{
  "network": "sepolia",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "contracts": {
    "csvlock": {
      "address": "${CSVLOCK_ADDR}",
      "tx_hash": "${CSVLOCK_TX}"
    }
  },
  "rpc_url": "${SEPOLIA_RPC_URL:0:30}...",
  "deployer": "${DEPLOYER_KEY:0:10}...${DEPLOYER_KEY: -8}"
}
EOF

echo -e "${GREEN}Deployment info saved to: ${DEPLOY_FILE}${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo -e "  1. Test contract: csv contract test --chain ethereum --network sepolia"
echo -e "  2. Update your application config to use the deployed contract address"
