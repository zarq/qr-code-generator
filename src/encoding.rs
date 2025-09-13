use crate::types::{DataMode, ErrorCorrection, Version};
use crate::ecc::generate_ecc as generate_reed_solomon_ecc;

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

fn generate_ecc(data_bits: &[u8], version: Version, error_correction: ErrorCorrection) -> Vec<u8> {
    // Convert bits to bytes
    let mut data_bytes = Vec::new();
    for chunk in data_bits.chunks(8) {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            byte |= bit << (7 - i);
        }
        data_bytes.push(byte);
    }
    
    // Get ECC codewords count based on version and error correction level
    let num_ecc_codewords = match (version, error_correction) {
        (Version::V3, ErrorCorrection::L) => 7,
        (Version::V3, ErrorCorrection::M) => 10,
        (Version::V3, ErrorCorrection::Q) => 13,
        (Version::V3, ErrorCorrection::H) => 17,
        _ => 10, // Default fallback
    };
    
    // Generate ECC using Reed-Solomon
    let ecc_bytes = generate_reed_solomon_ecc(&data_bytes, num_ecc_codewords);
    
    // Convert ECC bytes back to bits
    let mut ecc_bits = Vec::new();
    for byte in ecc_bytes {
        for i in 0..8 {
            ecc_bits.push((byte >> (7 - i)) & 1);
        }
    }
    
    ecc_bits
}
