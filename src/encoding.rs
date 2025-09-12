use crate::types::{DataMode, ErrorCorrection, Version};

pub struct EncodedData {
    pub data_bits: Vec<u8>,
    pub ecc_bits: Vec<u8>,
}

pub fn encode_data(url: &str, version: Version, error_correction: ErrorCorrection, mode: DataMode) -> EncodedData {
    let data_bits = match mode {
        DataMode::Byte => encode_byte(url, version),
        DataMode::Alphanumeric => encode_alphanumeric(url, version),
        _ => encode_byte(url, version), // Default to byte mode
    };
    let ecc_bits = generate_ecc(&data_bits, version, error_correction);
    
    EncodedData { data_bits, ecc_bits }
}

fn encode_byte(url: &str, _version: Version) -> Vec<u8> {
    let mut bits = Vec::new();
    
    // Mode indicator (4 bits) - Byte = 0100
    bits.extend_from_slice(&[0, 1, 0, 0]);
    
    // Character count (8 bits for Version 3)
    let count = url.len();
    for i in (0..8).rev() {
        bits.push(((count >> i) & 1) as u8);
    }
    
    // Encode each byte
    for byte in url.bytes() {
        for i in (0..8).rev() {
            bits.push(((byte >> i) & 1) as u8);
        }
    }
    
    bits
}

fn encode_alphanumeric(url: &str, _version: Version) -> Vec<u8> {
    let mut bits = Vec::new();
    
    // Mode indicator (4 bits) - Alphanumeric = 0010
    bits.extend_from_slice(&[0, 0, 1, 0]);
    
    // Character count (9 bits for Version 3)
    let count = url.len();
    for i in (0..9).rev() {
        bits.push(((count >> i) & 1) as u8);
    }
    
    // Encode character pairs
    let chars: Vec<char> = url.chars().collect();
    for chunk in chars.chunks(2) {
        if chunk.len() == 2 {
            let val1 = alphanumeric_value(chunk[0]);
            let val2 = alphanumeric_value(chunk[1]);
            let combined = val1 * 45 + val2;
            for i in (0..11).rev() {
                bits.push(((combined >> i) & 1) as u8);
            }
        } else {
            let val = alphanumeric_value(chunk[0]);
            for i in (0..6).rev() {
                bits.push(((val >> i) & 1) as u8);
            }
        }
    }
    
    bits
}

fn alphanumeric_value(c: char) -> u16 {
    match c {
        '0'..='9' => (c as u16) - ('0' as u16),
        'A'..='Z' => (c as u16) - ('A' as u16) + 10,
        ' ' => 36, '$' => 37, '%' => 38, '*' => 39, '+' => 40,
        '-' => 41, '.' => 42, '/' => 43, ':' => 44,
        _ => 0, // Invalid character, treat as 0
    }
}

fn generate_ecc(_data_bits: &[u8], _version: Version, _error_correction: ErrorCorrection) -> Vec<u8> {
    // Simplified ECC generation - just return zeros for now
    let ecc_length = 136; // 17 bytes * 8 bits for Version 3, Error Correction H
    vec![0; ecc_length]
}
