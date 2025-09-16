#!/bin/bash

# Test script for QR code structural patterns by version
# Usage: test_version_structure.sh <version_number>

if [ $# -ne 1 ]; then
    echo "Usage: $0 <version_number>"
    exit 1
fi

VERSION=$1
TEST_DATA=""

# Generate test data to force specific version (based on actual capacity testing)
case $VERSION in
    1) TEST_DATA="A" ;;
    2) TEST_DATA="ABCDEFGHIJKLMNOPQRSTUVWXYZ" ;;
    3) TEST_DATA="ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEF" ;;
    4) TEST_DATA="ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ" ;;
    5) TEST_DATA="ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFG" ;;
    6) TEST_DATA="ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ12345" ;;
    7) TEST_DATA="$(printf 'A%.0s' {1..122})" ;;
    8) TEST_DATA="$(printf 'A%.0s' {1..152})" ;;
    9) TEST_DATA="$(printf 'A%.0s' {1..180})" ;;
    10) TEST_DATA="$(printf 'A%.0s' {1..213})" ;;
    11) TEST_DATA="$(printf "%0603d" 1)" ;;
    12) TEST_DATA="$(printf "%0690d" 1)" ;;
    13) TEST_DATA="$(printf "%0795d" 1)" ;;
    14) TEST_DATA="$(printf "%0870d" 1)" ;;
    15) TEST_DATA="$(printf "%0990d" 1)" ;;
    16) TEST_DATA="$(printf "%01081d" 1)" ;;
    17) TEST_DATA="$(printf "%01211d" 1)" ;;
    18) TEST_DATA="$(printf "%01345d" 1)" ;;
    19) TEST_DATA="$(printf "%01499d" 1)" ;;
    20) TEST_DATA="$(printf "%01599d" 1)" ;;
    21) TEST_DATA="$(printf "%01707d" 1)" ;;
    22) TEST_DATA="$(printf "%01871d" 1)" ;;
    23) TEST_DATA="$(printf "%02058d" 1)" ;;
    24) TEST_DATA="$(printf "%02187d" 1)" ;;
    25) TEST_DATA="$(printf "%02394d" 1)" ;;
    26) TEST_DATA="$(printf "%02543d" 1)" ;;
    27) TEST_DATA="$(printf "%02700d" 1)" ;;
    28) TEST_DATA="$(printf "%02856d" 1)" ;;
    29) TEST_DATA="$(printf "%03034d" 1)" ;;
    30) TEST_DATA="$(printf "%03288d" 1)" ;;
    31) TEST_DATA="$(printf "%03485d" 1)" ;;
    32) TEST_DATA="$(printf "%03692d" 1)" ;;
    33) TEST_DATA="$(printf "%03908d" 1)" ;;
    34) TEST_DATA="$(printf "%04133d" 1)" ;;
    35) TEST_DATA="$(printf "%04342d" 1)" ;;
    36) TEST_DATA="$(printf "%04587d" 1)" ;;
    37) TEST_DATA="$(printf "%04774d" 1)" ;;
    38) TEST_DATA="$(printf "%05038d" 1)" ;;
    39) TEST_DATA="$(printf "%05312d" 1)" ;;
    40) TEST_DATA="$(printf "%05595d" 1)" ;;
    *) 
        echo "Version $VERSION not supported"
        exit 1
        ;;
esac

# Generate QR code
TEST_FILE="test_v${VERSION}_structure.png"
if [ $VERSION -le 10 ]; then
    ./target/debug/qr-generator --byte-mode --data "$TEST_DATA" -o "$TEST_FILE" >/dev/null 2>&1
else
    ./target/debug/qr-generator --numeric --data "$TEST_DATA" -o "$TEST_FILE" >/dev/null 2>&1
fi

if [ ! -f "$TEST_FILE" ]; then
    echo "FAIL: Could not generate QR code for V$VERSION"
    exit 1
fi

# Analyze structure
ANALYSIS=$(./target/debug/qr-analyzer "$TEST_FILE" 2>/dev/null)
if [ $? -ne 0 ]; then
    echo "FAIL: Could not analyze QR code for V$VERSION"
    rm -f "$TEST_FILE"
    exit 1
fi

# Extract key structural information
DETECTED_VERSION=$(echo "$ANALYSIS" | jq -r '.version_from_size // "null"')
SIZE=$(echo "$ANALYSIS" | jq -r '.size // "null"')
FINDER_COUNT=$(echo "$ANALYSIS" | jq '.finder_patterns | length')
FINDER_VALID=$(echo "$ANALYSIS" | jq '[.finder_patterns[] | select(.valid == true)] | length')
TIMING_VALID=$(echo "$ANALYSIS" | jq -r '.timing_patterns.valid')
DARK_MODULE=$(echo "$ANALYSIS" | jq -r '.dark_module.present')
ALIGNMENT_COUNT=$(echo "$ANALYSIS" | jq '.alignment_patterns | length')
ALIGNMENT_VALID=$(echo "$ANALYSIS" | jq '[.alignment_patterns[] | select(.valid == true)] | length')

# Validate structural patterns
ERRORS=0

if [ "$DETECTED_VERSION" != "V$VERSION" ]; then
    echo "FAIL: V$VERSION test - Expected version V$VERSION, got $DETECTED_VERSION"
    ERRORS=$((ERRORS + 1))
fi

if [ "$DETECTED_VERSION" = "null" ]; then
    echo "FAIL: V$VERSION test - Could not detect version"
    ERRORS=$((ERRORS + 1))
fi

if [ "$SIZE" = "null" ]; then
    echo "FAIL: V$VERSION test - Could not detect size"
    ERRORS=$((ERRORS + 1))
fi

if [ "$FINDER_COUNT" != "3" ]; then
    echo "FAIL: V$VERSION test - Expected 3 finder patterns, got $FINDER_COUNT"
    ERRORS=$((ERRORS + 1))
fi

if [ "$FINDER_VALID" != "3" ]; then
    echo "FAIL: V$VERSION test - Expected 3 valid finder patterns, got $FINDER_VALID"
    ERRORS=$((ERRORS + 1))
fi

if [ "$TIMING_VALID" != "true" ]; then
    echo "FAIL: V$VERSION test - Expected valid timing patterns, got $TIMING_VALID"
    ERRORS=$((ERRORS + 1))
fi

if [ "$DARK_MODULE" != "true" ]; then
    echo "FAIL: V$VERSION test - Expected dark module present, got $DARK_MODULE"
    ERRORS=$((ERRORS + 1))
fi

# For versions that should have alignment patterns, check they exist and at least some are valid
if [ $VERSION -ge 2 ] && [ "$ALIGNMENT_COUNT" -eq 0 ]; then
    echo "FAIL: V$VERSION test - Expected alignment patterns for V$VERSION, got none"
    ERRORS=$((ERRORS + 1))
fi

if [ $VERSION -ge 2 ] && [ "$ALIGNMENT_VALID" -eq 0 ]; then
    echo "FAIL: V$VERSION test - Expected valid alignment patterns for V$VERSION, got none valid"
    ERRORS=$((ERRORS + 1))
fi

# Clean up
rm -f "$TEST_FILE"

if [ $ERRORS -eq 0 ]; then
    echo "PASS: V$VERSION test ($DETECTED_VERSION generated) - All structural patterns valid"
    exit 0
else
    echo "FAIL: V$VERSION test ($DETECTED_VERSION generated) - $ERRORS structural validation errors"
    exit 1
fi
