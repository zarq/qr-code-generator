#!/usr/bin/env python3
"""
Generate Galois Field GF(256) lookup tables for QR code Reed-Solomon ECC.
Uses primitive polynomial x^8 + x^4 + x^3 + x^2 + 1 (0x11D)
"""

def generate_gf_tables():
    # GF(256) with primitive polynomial 0x11D
    primitive_poly = 0x11D
    
    # Generate EXP table (powers of 2)
    gf_exp = [0] * 256
    gf_log = [0] * 256
    
    x = 1
    for i in range(255):
        gf_exp[i] = x
        gf_log[x] = i
        x <<= 1
        if x & 0x100:
            x ^= primitive_poly
    
    return gf_exp, gf_log

def format_rust_array(name, arr):
    """Format array as Rust const array"""
    lines = [f"const {name}: [u8; 256] = ["]
    for i in range(0, 256, 16):
        chunk = arr[i:i+16]
        line = "    " + ", ".join(f"{x:3}" for x in chunk) + ","
        lines.append(line)
    lines.append("];")
    return "\n".join(lines)

if __name__ == "__main__":
    gf_exp, gf_log = generate_gf_tables()
    
    print("// Generated Galois Field GF(256) lookup tables")
    print("// Primitive polynomial: x^8 + x^4 + x^3 + x^2 + 1 (0x11D)")
    print()
    print(format_rust_array("GF_EXP", gf_exp))
    print()
    print(format_rust_array("GF_LOG", gf_log))
