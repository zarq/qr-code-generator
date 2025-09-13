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
    local format_copies_match=$(jq -r '.format_info.copies_match' "$GENERATED_DIR/${test_name}.json" 2>/dev/null)
    local dark_module_present=$(jq -r '.dark_module.present' "$GENERATED_DIR/${test_name}.json" 2>/dev/null)
    local timing_patterns_valid=$(jq -r '.timing_patterns.valid' "$GENERATED_DIR/${test_name}.json" 2>/dev/null)
    
    if [ "$status" = "success" ] && [ "$versions_match" = "true" ] && [ "$border_valid" = "true" ] && [ "$format_copies_match" = "true" ] && [ "$dark_module_present" = "true" ] && [ "$timing_patterns_valid" = "true" ]; then
        echo -e "${GREEN}PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}FAIL (validation)${NC}"
        echo "  Status: $status, Versions match: $versions_match, Border valid: $border_valid"
        echo "  Format copies match: $format_copies_match, Dark module: $dark_module_present, Timing patterns: $timing_patterns_valid"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Enhanced validation function for extracted data
check_extracted_data() {
    local file=$1
    local expected_data=$2
    
    local actual_data=$(jq -r '.data_analysis.extracted_data' "$file" 2>/dev/null)
    [ "$actual_data" = "$expected_data" ]
}

# Enhanced validation function for specific values
check_specific_value() {
    local file=$1
    local field=$2
    local expected=$3
    
    local actual=$(jq -r ".$field" "$file" 2>/dev/null)
    [ "$actual" = "$expected" ]
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

echo "=== Error Correction Validation Tests ==="

# Test that different ECC levels produce different format info
run_test "ecc_l_validation" "--numeric -u \"12345\" -l L" "ECC L validation"
run_test "ecc_m_validation" "--numeric -u \"12345\" -l M" "ECC M validation"
run_test "ecc_q_validation" "--numeric -u \"12345\" -l Q" "ECC Q validation"
run_test "ecc_h_validation" "--numeric -u \"12345\" -l H" "ECC H validation"

echo "=== Mask Pattern Validation Tests ==="

# Test that different mask patterns produce different format info
run_test "mask_validation_0" "--numeric -u \"123456\" --mask-pattern 0" "mask 0 validation"
run_test "mask_validation_1" "--numeric -u \"123456\" --mask-pattern 1" "mask 1 validation"
run_test "mask_validation_2" "--numeric -u \"123456\" --mask-pattern 2" "mask 2 validation"
run_test "mask_validation_3" "--numeric -u \"123456\" --mask-pattern 3" "mask 3 validation"

# Verify specific values are correctly set
echo "=== Specific Value Verification ==="

echo -n "Verifying ECC levels are correctly set... "
if check_specific_value "tests/generated/ecc_l_validation.json" "error_correction" "L" && \
   check_specific_value "tests/generated/ecc_m_validation.json" "error_correction" "M" && \
   check_specific_value "tests/generated/ecc_q_validation.json" "error_correction" "Q" && \
   check_specific_value "tests/generated/ecc_h_validation.json" "error_correction" "H"; then
    echo -e "${GREEN}PASS${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))

echo -n "Verifying mask patterns are correctly set... "
if check_specific_value "tests/generated/mask_validation_0.json" "mask_pattern" "Pattern0" && \
   check_specific_value "tests/generated/mask_validation_1.json" "mask_pattern" "Pattern1" && \
   check_specific_value "tests/generated/mask_validation_2.json" "mask_pattern" "Pattern2" && \
   check_specific_value "tests/generated/mask_validation_3.json" "mask_pattern" "Pattern3"; then
    echo -e "${GREEN}PASS${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))

echo "=== Data Extraction Validation Tests ==="

# Test that generated data matches extracted data
run_test "data_extract_numeric" "--numeric -u \"42\" -o tests/generated/data_extract_numeric.png" "numeric data extraction"
run_test "data_extract_byte" "--byte-mode -u \"Test123\" -o tests/generated/data_extract_byte.png" "byte data extraction"
run_test "data_extract_long" "--numeric -u \"9876543210\" -o tests/generated/data_extract_long.png" "long numeric extraction"

echo "=== Data Content Verification ==="

echo -n "Verifying numeric data extraction... "
if check_extracted_data "tests/generated/data_extract_numeric.json" "42"; then
    echo -e "${GREEN}PASS${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))

echo -n "Verifying byte data extraction... "
if check_extracted_data "tests/generated/data_extract_byte.json" "Test123"; then
    echo -e "${GREEN}PASS${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))

echo -n "Verifying long numeric data extraction... "
if check_extracted_data "tests/generated/data_extract_long.json" "9876543210"; then
    echo -e "${GREEN}PASS${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
TESTS_RUN=$((TESTS_RUN + 1))

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
