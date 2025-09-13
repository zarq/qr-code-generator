# QR Code Generator Test Suite

This directory contains a comprehensive test suite for the QR code generator and analyzer.

## Running Tests

From the project root:
```bash
./test.sh
```

Or directly:
```bash
./tests/run_tests.sh
```

## Test Categories

### Basic Functionality Tests
- Numeric mode encoding
- Alphanumeric mode encoding  
- Byte mode encoding

### Version Tests
- V1 (21x21) generation
- V3+ (larger) generation

### Error Correction Tests
- ECC levels L and M

### Mask Pattern Tests
- Specific mask patterns (0, 7)
- Skip mask functionality

### Numeric Encoding Tests
- Single digit encoding
- Two digit encoding
- Three digit encoding
- Long numeric strings

## Test Validation

Each test validates:
- ✅ `status == "success"`
- ✅ `versions_match == true` 
- ✅ `border_check.valid == true`
- ✅ Structural integrity (finder patterns, timing patterns)

## Generated Files

Test artifacts are stored in `tests/generated/` (gitignored):
- `*.png` - Generated QR code images
- `*.json` - Analyzer output for validation

## Adding New Tests

To add a new test, modify `run_tests.sh`:

```bash
run_test "test_name" "--your-args" "description"
```

The test framework automatically:
1. Generates QR code with specified arguments
2. Analyzes the generated QR code
3. Validates basic success criteria
4. Reports pass/fail status
