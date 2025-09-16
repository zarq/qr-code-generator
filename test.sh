#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

cd "$(dirname "$0")"

# Build the project
echo "Building project..."
cargo build --quiet
if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed${NC}"
    exit 1
fi

# Make test scripts executable
chmod +x tests/run_tests.sh
chmod +x tests/test_version_structure.sh

# Initialize counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -n "Testing $test_name... "
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command" >/dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}FAIL${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

# Run basic functionality tests
echo "=== Running Basic Functionality Tests ==="
./tests/run_tests.sh
if [ $? -eq 0 ]; then
    BASIC_PASSED=32
    BASIC_FAILED=0
else
    BASIC_PASSED=0
    BASIC_FAILED=32
fi

TOTAL_TESTS=$((TOTAL_TESTS + 32))
PASSED_TESTS=$((PASSED_TESTS + BASIC_PASSED))
FAILED_TESTS=$((FAILED_TESTS + BASIC_FAILED))

# Run structural pattern tests for all 40 versions
echo ""
echo "=== Running Structural Pattern Tests ==="
for version in {1..40}; do
    run_test "V$version structural patterns" "./tests/test_version_structure.sh $version"
done

# Print summary
echo ""
echo "=== Test Summary ==="
echo "Tests run: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
