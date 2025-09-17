#!/bin/bash

# Test qr-noise tool functionality
set -e

VERSION=${1:-10}

echo "Testing qr-noise with V$VERSION..."

# Generate test QR code
if [ $VERSION -le 10 ]; then
    case $VERSION in
        1) TEST_DATA="HELLO" ;;
        2) TEST_DATA="$(printf 'A%.0s' {1..20})" ;;
        3) TEST_DATA="$(printf 'A%.0s' {1..30})" ;;
        4) TEST_DATA="$(printf 'A%.0s' {1..50})" ;;
        5) TEST_DATA="$(printf 'A%.0s' {1..70})" ;;
        6) TEST_DATA="$(printf 'A%.0s' {1..90})" ;;
        7) TEST_DATA="$(printf 'A%.0s' {1..110})" ;;
        8) TEST_DATA="$(printf 'A%.0s' {1..130})" ;;
        9) TEST_DATA="$(printf 'A%.0s' {1..160})" ;;
        10) TEST_DATA="$(printf 'A%.0s' {1..213})" ;;
    esac
    ./target/debug/qr-generator --byte-mode --data "$TEST_DATA" -o "test_v${VERSION}.png" >/dev/null 2>&1
else
    case $VERSION in
        11) TEST_DATA="$(printf "%0603d" 1)" ;;
        13) TEST_DATA="$(printf "%0795d" 1)" ;;
        20) TEST_DATA="$(printf "%01599d" 1)" ;;
        *) echo "Version $VERSION not configured for test"; exit 1 ;;
    esac
    ./target/debug/qr-generator --numeric --data "$TEST_DATA" -o "test_v${VERSION}.png" >/dev/null 2>&1
fi

if [ ! -f "test_v${VERSION}.png" ]; then
    echo "FAIL: Could not generate test QR code for V$VERSION"
    exit 1
fi

# Test with 50% noise
./target/debug/qr-noise --input "test_v${VERSION}.png" --output "test_v${VERSION}_noisy.png" --percentage 50 >/dev/null 2>&1

if [ ! -f "test_v${VERSION}_noisy.png" ]; then
    echo "FAIL: Could not generate noisy QR code for V$VERSION"
    exit 1
fi

# Analyze the noisy QR code
ANALYSIS=$(./target/debug/qr-analyzer "test_v${VERSION}_noisy.png" 2>/dev/null)

# Check that functional patterns are still valid
FINDER_VALID=$(echo "$ANALYSIS" | jq -r '.finder_patterns | map(.valid) | all')
TIMING_VALID=$(echo "$ANALYSIS" | jq -r '.timing_patterns.valid')

# For V2+, check alignment patterns
if [ $VERSION -gt 1 ]; then
    ALIGNMENT_VALID=$(echo "$ANALYSIS" | jq -r '.alignment_patterns | map(.valid) | all')
else
    ALIGNMENT_VALID="true"
fi

# Check version detection
VERSION_DETECTED=$(echo "$ANALYSIS" | jq -r '.version_from_size')
EXPECTED_VERSION="V$VERSION"

# Clean up
rm -f "test_v${VERSION}.png" "test_v${VERSION}_noisy.png"

# Report results
if [ "$FINDER_VALID" = "true" ] && [ "$TIMING_VALID" = "true" ] && [ "$ALIGNMENT_VALID" = "true" ] && [ "$VERSION_DETECTED" = "$EXPECTED_VERSION" ]; then
    echo "PASS: V$VERSION noise test - All functional patterns preserved"
    exit 0
else
    echo "FAIL: V$VERSION noise test - Functional patterns corrupted"
    echo "  Finder patterns valid: $FINDER_VALID"
    echo "  Timing patterns valid: $TIMING_VALID" 
    echo "  Alignment patterns valid: $ALIGNMENT_VALID"
    echo "  Version detected: $VERSION_DETECTED (expected: $EXPECTED_VERSION)"
    exit 1
fi
