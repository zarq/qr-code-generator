use crate::types::{DataMode, ErrorCorrection, Version};

pub struct EncodedData {
    pub data_bits: Vec<u8>,
    pub ecc_bits: Vec<u8>,
}

pub fn encode_data(url: &str, version: Version, error_correction: ErrorCorrection) -> EncodedData {
    let data_bits = encode_alphanumeric(url, version);
    let ecc_bits = generate_ecc(&data_bits, version, error_correction);
    
    EncodedData { data_bits, ecc_bits }
}

fn encode_alphanumeric(url: &str, version: Version) -> Vec<u8> {
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

fn generate_ecc(data_bits: &[u8], version: Version, error_correction: ErrorCorrection) -> Vec<u8> {
    // Simplified ECC generation - just return zeros for now
    let ecc_length = match (version, error_correction) {
        (Version::V3, ErrorCorrection::H) => 136, // 17 bytes * 8 bits
        _ => 136,
    };
    vec![0; ecc_length]
}
