use crate::capacity::get_data_capacity_in_bits;
use crate::types::{DataMode, ErrorCorrection, Version};
use crate::ecc::generate_ecc as generate_reed_solomon_ecc;

pub struct EncodedData {
    pub data_bits: Vec<u8>,
    pub ecc_bits: Vec<u8>,
}

pub fn encode_data(data: &str, version: Version, error_correction: ErrorCorrection, mode: DataMode) -> EncodedData {
    let mut data_bits = match mode {
        DataMode::Numeric => encode_numeric(data, version),
        DataMode::Byte => encode_byte(data, version),
        DataMode::Alphanumeric => encode_alphanumeric(data, version),
    };
    
    // Add padding to reach required data capacity
    add_padding(&mut data_bits, version, error_correction);
    
    let ecc_bits = generate_ecc(&data_bits, version, error_correction);
    
    EncodedData { data_bits, ecc_bits }
}

fn add_padding(data_bits: &mut Vec<u8>, version: Version, error_correction: ErrorCorrection) {
    // Get data capacity in bits
    let data_capacity_bits = get_data_capacity_in_bits(version, error_correction);
    
    // Add terminator (up to 4 zero bits, only if there's space)
    if data_bits.len() < data_capacity_bits {
        let terminator_bits = std::cmp::min(4, data_capacity_bits - data_bits.len());
        data_bits.extend(vec![0; terminator_bits]);
    }
    
    // Pad to byte boundary
    while data_bits.len() % 8 != 0 && data_bits.len() < data_capacity_bits {
        data_bits.push(0);
    }
    
    // Add padding bytes (0xEC, 0x11 alternating)
    let mut padding_byte = 0xEC;
    while data_bits.len() < data_capacity_bits {
        for i in 0..8 {
            if data_bits.len() < data_capacity_bits {
                data_bits.push((padding_byte >> (7 - i)) & 1);
            }
        }
        padding_byte = if padding_byte == 0xEC { 0x11 } else { 0xEC };
    }
}

fn encode_numeric(data: &str, _version: Version) -> Vec<u8> {
    let mut bits = Vec::new();
    
    // Mode indicator (4 bits) - Numeric = 0001
    bits.extend_from_slice(&[0, 0, 0, 1]);
    
    // Character count (10 bits for Version 3)
    let count = data.len();
    for i in (0..10).rev() {
        bits.push(((count >> i) & 1) as u8);
    }
    
    // Encode digits in groups of 3
    let digits: Vec<char> = data.chars().collect();
    for chunk in digits.chunks(3) {
        match chunk.len() {
            3 => {
                let val = chunk[0].to_digit(10).unwrap() * 100 + 
                         chunk[1].to_digit(10).unwrap() * 10 + 
                         chunk[2].to_digit(10).unwrap();
                for i in (0..10).rev() {
                    bits.push(((val >> i) & 1) as u8);
                }
            }
            2 => {
                let val = chunk[0].to_digit(10).unwrap() * 10 + 
                         chunk[1].to_digit(10).unwrap();
                for i in (0..7).rev() {
                    bits.push(((val >> i) & 1) as u8);
                }
            }
            1 => {
                let val = chunk[0].to_digit(10).unwrap();
                for i in (0..4).rev() {
                    bits.push(((val >> i) & 1) as u8);
                }
            }
            _ => {}
        }
    }
    
    bits
}

fn encode_byte(data: &str, _version: Version) -> Vec<u8> {
    let mut bits = Vec::new();
    
    // Mode indicator (4 bits) - Byte = 0100
    bits.extend_from_slice(&[0, 1, 0, 0]);
    
    // Character count (8 bits for Version 3)
    let count = data.len();
    for i in (0..8).rev() {
        bits.push(((count >> i) & 1) as u8);
    }
    
    // Encode each byte
    for byte in data.bytes() {
        for i in (0..8).rev() {
            bits.push(((byte >> i) & 1) as u8);
        }
    }
    
    bits
}

fn encode_alphanumeric(data: &str, _version: Version) -> Vec<u8> {
    let mut bits = Vec::new();
    
    // Mode indicator (4 bits) - Alphanumeric = 0010
    bits.extend_from_slice(&[0, 0, 1, 0]);
    
    // Character count (9 bits for Version 3)
    let count = data.len();
    for i in (0..9).rev() {
        bits.push(((count >> i) & 1) as u8);
    }
    
    // Encode character pairs
    let chars: Vec<char> = data.chars().collect();
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
    // Get block structure info
    let (num_blocks_group1, data_codewords_group1, num_blocks_group2, data_codewords_group2, ecc_codewords_per_block) = 
        get_block_info(version, error_correction);
    
    // Convert bits to bytes
    let mut data_bytes = Vec::new();
    for chunk in data_bits.chunks(8) {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            byte |= bit << (7 - i);
        }
        data_bytes.push(byte);
    }
    
    // Split data into blocks
    let mut data_blocks = Vec::new();
    let mut byte_index = 0;
    
    // Group 1 blocks
    for _ in 0..num_blocks_group1 {
        let block_size = data_codewords_group1;
        let block = if byte_index + block_size <= data_bytes.len() {
            data_bytes[byte_index..byte_index + block_size].to_vec()
        } else {
            let mut block = data_bytes[byte_index..].to_vec();
            block.resize(block_size, 0xEC); // Pad with standard padding
            block
        };
        data_blocks.push(block);
        byte_index += block_size;
    }
    
    // Group 2 blocks
    for _ in 0..num_blocks_group2 {
        let block_size = data_codewords_group2;
        let block = if byte_index + block_size <= data_bytes.len() {
            data_bytes[byte_index..byte_index + block_size].to_vec()
        } else {
            let mut block = data_bytes[byte_index..].to_vec();
            block.resize(block_size, 0xEC); // Pad with standard padding
            block
        };
        data_blocks.push(block);
        byte_index += block_size;
    }
    
    // Generate ECC for each block
    let mut ecc_blocks = Vec::new();
    for data_block in &data_blocks {
        let ecc_block = generate_reed_solomon_ecc(data_block, ecc_codewords_per_block);
        ecc_blocks.push(ecc_block);
    }
    
    // Print verbose block information
    if std::env::args().any(|arg| arg == "--verbose" || arg == "-V") {
        println!("\n=== Block Structure ===");
        println!("Group 1: {} blocks of {} data codewords each", num_blocks_group1, data_codewords_group1);
        if num_blocks_group2 > 0 {
            println!("Group 2: {} blocks of {} data codewords each", num_blocks_group2, data_codewords_group2);
        }
        println!("ECC codewords per block: {}", ecc_codewords_per_block);
        
        for (i, block) in data_blocks.iter().enumerate() {
            println!("Data Block {}: {} bytes", i + 1, block.len());
            println!("  Hex: {}", block.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
        }
        
        for (i, block) in ecc_blocks.iter().enumerate() {
            println!("ECC Block {}: {} bytes", i + 1, block.len());
            println!("  Hex: {}", block.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
        }
    }
    
    // Interleave and convert back to bits
    let mut all_ecc_bits = Vec::new();
    
    // Interleave ECC blocks byte by byte
    let max_ecc_bytes = ecc_blocks.iter().map(|b| b.len()).max().unwrap_or(0);
    for byte_index in 0..max_ecc_bytes {
        for block in &ecc_blocks {
            if byte_index < block.len() {
                // Convert byte to bits
                for bit_pos in 0..8 {
                    all_ecc_bits.push((block[byte_index] >> (7 - bit_pos)) & 1);
                }
            }
        }
    }
    
    all_ecc_bits
}

fn get_block_info(version: Version, error_correction: ErrorCorrection) -> (usize, usize, usize, usize, usize) {
    // Returns: (num_blocks_group1, data_codewords_group1, num_blocks_group2, data_codewords_group2, ecc_codewords_per_block)
    match (version, error_correction) {
        // Version 1
        (Version::V1, ErrorCorrection::L) => (1, 19, 0, 0, 7),
        (Version::V1, ErrorCorrection::M) => (1, 16, 0, 0, 10),
        (Version::V1, ErrorCorrection::Q) => (1, 13, 0, 0, 13),
        (Version::V1, ErrorCorrection::H) => (1, 9, 0, 0, 17),
        // Version 2
        (Version::V2, ErrorCorrection::L) => (1, 34, 0, 0, 10),
        (Version::V2, ErrorCorrection::M) => (1, 28, 0, 0, 16),
        (Version::V2, ErrorCorrection::Q) => (1, 22, 0, 0, 22),
        (Version::V2, ErrorCorrection::H) => (1, 16, 0, 0, 28),
        // Version 3
        (Version::V3, ErrorCorrection::L) => (1, 55, 0, 0, 15),
        (Version::V3, ErrorCorrection::M) => (1, 44, 0, 0, 26),
        (Version::V3, ErrorCorrection::Q) => (2, 17, 0, 0, 18),
        (Version::V3, ErrorCorrection::H) => (2, 13, 0, 0, 22),
        // Version 4
        (Version::V4, ErrorCorrection::L) => (1, 80, 0, 0, 20),
        (Version::V4, ErrorCorrection::M) => (2, 32, 0, 0, 18),
        (Version::V4, ErrorCorrection::Q) => (2, 24, 0, 0, 26),
        (Version::V4, ErrorCorrection::H) => (4, 9, 0, 0, 16),
        _ => (1, 16, 0, 0, 10), // Default fallback
    }
}
