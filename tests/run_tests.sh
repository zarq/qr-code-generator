#!/bin/bash

# QR Code Generator Test Suite
set -e

cd "$(dirname "$0")/.."

GENERATOR="./target/debug/qr-generator"
ANALYZER="./target/debug/qr-analyzer"
GENERATED_DIR="tests/generated"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Ensure binaries are built
echo "Building binaries..."
$HOME/.cargo/bin/cargo build --quiet

# Clean generated directory
mkdir -p $GENERATED_DIR
rm -f $GENERATED_DIR/*

# Test helper function
run_test() {
    local test_name=$1
    local generator_args="$2"
    local description="$3"
    
    TESTS_RUN=$((TESTS_RUN + 1))
    echo -n "Testing $description... "
    
    # Generate QR code
    if ! eval "$GENERATOR $generator_args -o $GENERATED_DIR/${test_name}.png" >/dev/null 2>&1; then
        echo -e "${RED}FAIL (generation)${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return
    fi
    
    # Analyze QR code
    if ! $ANALYZER "$GENERATED_DIR/${test_name}.png" > "$GENERATED_DIR/${test_name}.json" 2>/dev/null; then
        echo -e "${RED}FAIL (analysis)${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return
    fi
    
    # Check basic success criteria
    local status=$(jq -r '.status' "$GENERATED_DIR/${test_name}.json" 2>/dev/null)
    local versions_match=$(jq -r '.versions_match' "$GENERATED_DIR/${test_name}.json" 2>/dev/null)
    local border_valid=$(jq -r '.border_check.valid' "$GENERATED_DIR/${test_name}.json" 2>/dev/null)
    
    if [ "$status" = "success" ] && [ "$versions_match" = "true" ] && [ "$border_valid" = "true" ]; then
        echo -e "${GREEN}PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}FAIL (validation)${NC}"
        echo "  Status: $status, Versions match: $versions_match, Border valid: $border_valid"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Run tests
echo "=== Basic Functionality Tests ==="

run_test "numeric_basic" "--numeric -u \"123456\"" "numeric mode"
run_test "alphanumeric_basic" "--alphanumeric-mode -u \"HELLO123\"" "alphanumeric mode"  
run_test "byte_basic" "--byte-mode -u \"Hello World\"" "byte mode"

echo "=== Version Tests ==="

run_test "version_v1" "--numeric -u \"123\"" "V1 generation"
run_test "version_v3" "--byte-mode -u \"This is a longer text string to force version 3\"" "V3 generation"

echo "=== Error Correction Tests ==="

run_test "ecc_l" "--numeric -u \"12345\" -l L" "ECC level L"
run_test "ecc_m" "--numeric -u \"12345\" -l M" "ECC level M"

echo "=== Mask Pattern Tests ==="

run_test "mask_0" "--numeric -u \"123456\" --mask-pattern 0" "mask pattern 0"
run_test "mask_7" "--numeric -u \"123456\" --mask-pattern 7" "mask pattern 7"
run_test "skip_mask" "--numeric -u \"123456\" --skip-mask" "skip mask"

echo "=== Numeric Encoding Tests ==="

run_test "numeric_single" "--numeric -u \"7\"" "single digit"
run_test "numeric_double" "--numeric -u \"42\"" "two digits"
run_test "numeric_triple" "--numeric -u \"789\"" "three digits"
run_test "numeric_long" "--numeric -u \"1234567890\"" "long numeric"

# Summary
echo ""
echo "=== Test Summary ==="
echo "Tests run: $TESTS_RUN"
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed.${NC}"
    exit 1
fi
