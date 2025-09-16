#!/bin/bash

echo "=== Testing Alignment Patterns for All QR Versions ==="

# Build the analyzer
cargo build --quiet

# Test data for different versions (increasing length to force higher versions)
declare -a test_data=(
    "1"                                                                                                                     # V1
    "12345678901234567890123456789012345"                                                                                   # V2
    "This is a longer text that should force version 3 with more data capacity than version 2"                           # V3
    "This is an even longer text that should definitely force version 4 with significantly more data capacity than V3"   # V4
    "This is a much longer text that should force version 5 with even more data capacity than version 4 and previous versions" # V5
)

# Test first 5 versions
for i in {0..4}; do
    version=$((i + 1))
    echo "Testing V$version..."
    
    ./target/debug/qr-generator --byte-mode --data "${test_data[$i]}" -o "test_v$version.png" 2>/dev/null
    
    if [ -f "test_v$version.png" ]; then
        result=$(./target/debug/qr-analyzer "test_v$version.png" | jq -r '.version_from_size // "null"')
        alignment_count=$(./target/debug/qr-analyzer "test_v$version.png" | jq '.alignment_patterns | length')
        valid_count=$(./target/debug/qr-analyzer "test_v$version.png" | jq '[.alignment_patterns[] | select(.valid == true)] | length')
        
        echo "  Generated: $result, Alignment patterns: $alignment_count, Valid: $valid_count"
        rm "test_v$version.png"
    else
        echo "  Failed to generate QR code"
    fi
done

echo "=== Alignment Pattern Test Complete ==="
