#!/bin/bash

cd "$(dirname "$0")/.."

echo "Running simple test..."

# Generate QR code
./target/debug/qr-generator --numeric -u "123" -o tests/generated/simple.png

# Analyze it
./target/debug/qr-analyzer tests/generated/simple.png > tests/generated/simple.json

# Check status
STATUS=$(jq -r '.status' tests/generated/simple.json)
VERSIONS_MATCH=$(jq -r '.versions_match' tests/generated/simple.json)

echo "Status: $STATUS"
echo "Versions match: $VERSIONS_MATCH"

if [ "$STATUS" = "success" ] && [ "$VERSIONS_MATCH" = "true" ]; then
    echo "✓ Test passed"
else
    echo "✗ Test failed"
fi
