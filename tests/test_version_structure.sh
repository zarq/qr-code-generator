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
    7) TEST_DATA="ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ123456789" ;;
    *) 
        # For higher versions, use numeric mode
        NUMERIC_CAPACITY=$((20 + VERSION * 15))
        TEST_DATA=$(printf "%0${NUMERIC_CAPACITY}d" 1)
        ;;
esac

# Generate QR code
TEST_FILE="test_v${VERSION}_structure.png"
if [ $VERSION -le 7 ]; then
    ./target/debug/qr-generator --byte-mode --data "$TEST_DATA" -o "$TEST_FILE" >/dev/null 2>&1
else
    ./target/debug/qr-generator --numeric-mode --data "$TEST_DATA" -o "$TEST_FILE" >/dev/null 2>&1
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
