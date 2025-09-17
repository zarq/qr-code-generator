#!/bin/bash

# Test qr-noise tool across multiple QR code versions
set -e

echo "Testing qr-noise tool functionality..."

PASSED=0
FAILED=0

for VERSION in 1 2 5 7 10 11 13 20; do
    echo -n "Testing V$VERSION... "
    
    if ./tests/test_qr_noise.sh $VERSION >/dev/null 2>&1; then
        echo "PASS"
        PASSED=$((PASSED + 1))
    else
        echo "FAIL"
        FAILED=$((FAILED + 1))
    fi
done

echo
echo "=== QR-Noise Test Summary ==="
echo "Versions tested: 8"
echo "Passed: $PASSED"
echo "Failed: $FAILED"

if [ $FAILED -eq 0 ]; then
    echo "All qr-noise tests passed!"
    exit 0
else
    echo "Some qr-noise tests failed!"
    exit 1
fi
