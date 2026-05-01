#!/bin/bash
# CSV Adapter Production Surface Audit Script
# Phase 1 of Production Guarantee Plan
#
# This script scans the codebase for forbidden markers in production code:
# - TODO, FIXME
# - placeholder, stub, mock
# - simulation, simulate
# - unimplemented!, todo!
# - Fake tx/proof/hash generation patterns
#
# Usage: ./scripts/audit-production-surface.sh
# Exit code: 0 if no production findings, 1 if violations found

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Forbidden patterns to search for
FORBIDDEN_PATTERNS=(
    "TODO"
    "FIXME"
    "placeholder"
    "stub"
    "mock"
    "simulation"
    "simulate"
    "unimplemented!"
    "todo!"
)

# Patterns that suggest fake/placeholder implementations
FAKE_PATTERNS=(
    "fake_hash"
    "fake_tx"
    "fake_proof"
    "placeholder_hash"
    "placeholder_tx"
    "placeholder_proof"
    "dummy_hash"
    "dummy_tx"
    "dummy_proof"
    "0x0000000000000000000000000000000000000000000000000000000000000000"
)

# Excluded paths (tests, examples, docs, scripts)
EXCLUDED_PATHS=(
    "/tests/"
    "#\[cfg(test)\]"
    "/examples/"
    "/docs/"
    "/scripts/"
    "test_"
    "_test.rs"
    "tests.rs"
    ".md"
    "// .*TODO"  # Comments mentioning TODO
    "// .*FIXME"
)

VIOLATIONS_FILE=$(mktemp)
trap "rm -f $VIOLATIONS_FILE" EXIT

echo "========================================"
echo "CSV Adapter Production Surface Audit"
echo "========================================"
echo ""
echo "Scanning for forbidden markers in production code..."
echo ""

# Function to check if a file is excluded
is_excluded() {
    local file="$1"
    for pattern in "${EXCLUDED_PATHS[@]}"; do
        if [[ "$file" == *"$pattern"* ]]; then
            return 0
        fi
    done
    return 1
}

# Function to scan a file for forbidden patterns
scan_file() {
    local file="$1"
    local relative_file="${file#$PROJECT_ROOT/}"

    # Skip excluded files
    if is_excluded "$file"; then
        return 0
    fi

    local found_violation=false
    local file_violations=""

    # Check for each forbidden pattern
    for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
        # Use grep with line numbers, but exclude comments that just mention the words
        local matches
        matches=$(grep -n "$pattern" "$file" 2>/dev/null || true)

        if [[ -n "$matches" ]]; then
            # Filter out pure comment lines that are just documentation
            while IFS= read -r line; do
                # Skip if line is only a comment explaining the pattern
                if [[ "$line" =~ ^[[:space:]]*//.*$ ]]; then
                    continue
                fi
                # Skip if it's in a string literal describing the pattern
                if [[ "$line" =~ \".*${pattern}.*\" ]]; then
                    continue
                fi

                if [[ -n "$line" ]]; then
                    found_violation=true
                    file_violations+="  - Line $line: contains '$pattern'"
                    file_violations+=$'\n'
                fi
            done <<< "$matches"
        fi
    done

    # Check for fake patterns
    for pattern in "${FAKE_PATTERNS[@]}"; do
        local matches
        matches=$(grep -n "$pattern" "$file" 2>/dev/null || true)

        if [[ -n "$matches" ]]; then
            while IFS= read -r line; do
                if [[ -n "$line" ]]; then
                    found_violation=true
                    file_violations+="  - Line $line: contains fake/placeholder pattern"
                    file_violations+=$'\n'
                fi
            done <<< "$matches"
        fi
    done

    if [[ "$found_violation" == true ]]; then
        echo "$relative_file" >> "$VIOLATIONS_FILE"
        echo "$file_violations" >> "$VIOLATIONS_FILE"
        echo "" >> "$VIOLATIONS_FILE"
    fi
}

# Find all Rust files and scan them
export -f scan_file is_excluded
export PROJECT_ROOT VIOLATIONS_FILE FORBIDDEN_PATTERNS FAKE_PATTERNS EXCLUDED_PATHS

# Use process substitution instead of pipe to avoid subshell
# This ensures exit codes propagate correctly
while read -r file; do
    scan_file "$file"
done < <(find "$PROJECT_ROOT" -name "*.rs" -type f)

# Count violations
VIOLATION_COUNT=$(grep -c '^[^-]' "$VIOLATIONS_FILE" 2>/dev/null || echo "0")
if [[ -s "$VIOLATIONS_FILE" ]]; then
    VIOLATION_COUNT=$(grep -v '^  -' "$VIOLATIONS_FILE" | grep -v '^$' | wc -l)
else
    VIOLATION_COUNT=0
fi

echo "========================================"
echo "AUDIT RESULTS"
echo "========================================"
echo ""

if [[ -s "$VIOLATIONS_FILE" ]]; then
    echo -e "${RED}FAIL: Production code contains forbidden markers${NC}"
    echo ""
    echo "Files with violations:"
    cat "$VIOLATIONS_FILE"
    echo ""
    echo -e "${YELLOW}Total files with violations: $VIOLATION_COUNT${NC}"
    echo ""
    echo "Forbidden markers found:"
    echo "  - TODO, FIXME"
    echo "  - placeholder, stub, mock"
    echo "  - simulation, simulate"
    echo "  - unimplemented!, todo!"
    echo "  - Fake/placeholder hash, tx, proof patterns"
    echo ""
    echo "These markers are only allowed in:"
    echo "  - tests/ directories"
    echo "  - #[cfg(test)] modules"
    echo "  - examples/ (non-production)"
    echo "  - documentation describing patterns"
    echo ""
    echo "To resolve:"
    echo "  1. Remove the marker and implement the functionality"
    echo "  2. Move test-only code to #[cfg(test)] modules"
    echo "  3. Replace unimplemented! with proper error handling"
    echo ""
    exit 1
else
    echo -e "${GREEN}PASS: No forbidden markers found in production code${NC}"
    echo ""
    echo "All scanned files comply with production standards."
    echo ""
    exit 0
fi
