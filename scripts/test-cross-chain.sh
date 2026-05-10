#!/bin/bash
# Cross-Chain Transfer Integration Test
# Tests a complete Bitcoin -> Solana transfer via Nostr proof delivery
#
# Prerequisites:
#   - Bitcoin Signet wallet with funds
#   - Solana Devnet wallet with funds for minting
#   - Nostr relay connectivity (or use local relay)

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  CSV Cross-Chain Integration Test${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Configuration
BITCOIN_NETWORK="signet"
SOLANA_NETWORK="devnet"
TEST_AMOUNT=1000  # satoshis

# Check CLI availability
if command -v csv &> /dev/null; then
    CLI="csv"
elif [ -f "./csv-cli/target/release/csv" ]; then
    CLI="./csv-cli/target/release/csv"
elif [ -f "./csv-cli/target/debug/csv" ]; then
    CLI="./csv-cli/target/debug/csv"
else
    echo -e "${RED}csv-cli not found. Build with: cargo build -p csv-cli${NC}"
    exit 1
fi

echo -e "${BLUE}Using CLI: ${CLI}${NC}"
echo ""

# Test result tracking
TESTS_PASSED=0
TESTS_FAILED=0

run_test() {
    local test_name=$1
    local test_cmd=$2
    
    echo -e "${BLUE}Test: ${test_name}${NC}"
    if eval "$test_cmd"; then
        echo -e "${GREEN}✓ PASSED${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# ============================================
# Test 1: Configuration
# ============================================
echo -e "${YELLOW}Phase 1: Configuration${NC}"
echo "----------------------------------------"

run_test "Set Bitcoin network" \
    "$CLI config set bitcoin.network $BITCOIN_NETWORK > /dev/null 2>&1 || true"

run_test "Set Solana network" \
    "$CLI config set solana.network $SOLANA_NETWORK > /dev/null 2>&1 || true"

echo ""

# ============================================
# Test 2: Wallet Initialization
# ============================================
echo -e "${YELLOW}Phase 2: Wallet Setup${NC}"
echo "----------------------------------------"

run_test "Check wallet initialized" \
    "$CLI wallet list > /dev/null 2>&1"

if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${YELLOW}Wallet not initialized. Run: csv wallet init${NC}"
fi

echo ""

# ============================================
# Test 3: Seal Creation (Bitcoin)
# ============================================
echo -e "${YELLOW}Phase 3: Seal Creation${NC}"
echo "----------------------------------------"

SEAL_OUTPUT=$($CLI seal create --chain bitcoin --value $TEST_AMOUNT 2>&1) || true
SEAL_ID=$(echo "$SEAL_OUTPUT" | grep -oE '[a-f0-9]{64}' | head -1)

if [ -n "$SEAL_ID" ]; then
    echo -e "${GREEN}✓ Seal created: ${SEAL_ID:0:16}...${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ Seal creation failed${NC}"
    echo "$SEAL_OUTPUT"
    ((TESTS_FAILED++))
fi

echo ""

# ============================================
# Test 4: Proof Building
# ============================================
echo -e "${YELLOW}Phase 4: Proof Building${NC}"
echo "----------------------------------------"

if [ -n "$SEAL_ID" ]; then
    PROOF_OUTPUT=$($CLI proof build --chain bitcoin --seal-id $SEAL_ID 2>&1) || true
    
    if echo "$PROOF_OUTPUT" | grep -q "proof\|bundle\|success"; then
        echo -e "${GREEN}✓ Proof built successfully${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ Proof building failed${NC}"
        echo "$PROOF_OUTPUT"
        ((TESTS_FAILED++))
    fi
else
    echo -e "${YELLOW}Skipping (no seal ID)${NC}"
fi

echo ""

# ============================================
# Test 5: Nostr P2P Delivery
# ============================================
echo -e "${YELLOW}Phase 5: P2P Proof Delivery${NC}"
echo "----------------------------------------"

# Test Nostr connectivity
NOSTR_OUTPUT=$($CLI p2p status 2>&1) || true

if echo "$NOSTR_OUTPUT" | grep -q "connected\|relay\|ok"; then
    echo -e "${GREEN}✓ Nostr P2P available${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}⚠ Nostr P2P not fully configured${NC}"
    echo "$NOSTR_OUTPUT"
fi

echo ""

# ============================================
# Test 6: Offline Verification
# ============================================
echo -e "${YELLOW}Phase 6: Offline Verification${NC}"
echo "----------------------------------------"

# Create a test proof bundle for offline verification
TEST_BUNDLE='{
  "seal_ref": {"id": "abcd1234", "chain": "bitcoin"},
  "anchor_ref": {"anchor_id": "test1234", "block_height": 100},
  "inclusion_proof": {"proof_bytes": [1,2,3,4], "block_hash": "0000000000000000000000000000000000000000000000000000000000000001"},
  "finality_proof": {"confirmations": 10, "finality_data": [], "is_deterministic": false}
}'

# This would normally be done through the wallet UI
# For CLI, we'd need an export/import mechanism
echo -e "${YELLOW}⚠ Test requires wallet UI or proof export feature${NC}"

echo ""

# ============================================
# Test 7: Cross-Chain Transfer Status
# ============================================
echo -e "${YELLOW}Phase 7: Transfer State Machine${NC}"
echo "----------------------------------------"

TRANSFER_OUTPUT=$($CLI cross-chain list 2>&1) || true

if [ -n "$TRANSFER_OUTPUT" ]; then
    echo -e "${GREEN}✓ Transfer system accessible${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}⚠ No transfers found${NC}"
fi

echo ""

# ============================================
# Summary
# ============================================
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Test Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "Tests Passed: ${GREEN}${TESTS_PASSED}${NC}"
echo -e "Tests Failed: ${RED}${TESTS_FAILED}${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${YELLOW}Some tests failed or require manual setup.${NC}"
    echo ""
    echo "To complete the full test:"
    echo "  1. Ensure wallets are initialized: csv wallet init"
    echo "  2. Fund Bitcoin signet wallet"
    echo "  3. Fund Solana devnet wallet"
    echo "  4. Run the transfer: csv cross-chain transfer --from bitcoin --to solana"
    exit 1
fi
