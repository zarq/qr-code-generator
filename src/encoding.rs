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
    let data_capacity_bits = get_data_capacity_bits(version, error_correction);
    
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

fn get_data_capacity_bits(version: Version, error_correction: ErrorCorrection) -> usize {
    // Data capacity in bits for different versions and ECC levels
    // Source: https://www.thonky.com/qr-code-tutorial/character-capacities
    match (version, error_correction) {
        // Version 1
        (Version::V1, ErrorCorrection::L) => 152,
        (Version::V1, ErrorCorrection::M) => 128,
        (Version::V1, ErrorCorrection::Q) => 104,
        (Version::V1, ErrorCorrection::H) => 72,
        // Version 2
        (Version::V2, ErrorCorrection::L) => 272,
        (Version::V2, ErrorCorrection::M) => 224,
        (Version::V2, ErrorCorrection::Q) => 176,
        (Version::V2, ErrorCorrection::H) => 128,
        // Version 3
        (Version::V3, ErrorCorrection::L) => 440,
        (Version::V3, ErrorCorrection::M) => 352,
        (Version::V3, ErrorCorrection::Q) => 272,
        (Version::V3, ErrorCorrection::H) => 208,
        // Version 4
        (Version::V4, ErrorCorrection::L) => 640,
        (Version::V4, ErrorCorrection::M) => 512,
        (Version::V4, ErrorCorrection::Q) => 384,
        (Version::V4, ErrorCorrection::H) => 288,
        // Version 5
        (Version::V5, ErrorCorrection::L) => 864,
        (Version::V5, ErrorCorrection::M) => 688,
        (Version::V5, ErrorCorrection::Q) => 496,
        (Version::V5, ErrorCorrection::H) => 368,
        // Version 6
        (Version::V6, ErrorCorrection::L) => 1088,
        (Version::V6, ErrorCorrection::M) => 864,
        (Version::V6, ErrorCorrection::Q) => 608,
        (Version::V6, ErrorCorrection::H) => 480,
        // Version 7
        (Version::V7, ErrorCorrection::L) => 1248,
        (Version::V7, ErrorCorrection::M) => 992,
        (Version::V7, ErrorCorrection::Q) => 704,
        (Version::V7, ErrorCorrection::H) => 528,
        // Version 8
        (Version::V8, ErrorCorrection::L) => 1552,
        (Version::V8, ErrorCorrection::M) => 1232,
        (Version::V8, ErrorCorrection::Q) => 880,
        (Version::V8, ErrorCorrection::H) => 688,
        // Version 9
        (Version::V9, ErrorCorrection::L) => 1856,
        (Version::V9, ErrorCorrection::M) => 1456,
        (Version::V9, ErrorCorrection::Q) => 1056,
        (Version::V9, ErrorCorrection::H) => 800,
        // Version 10
        (Version::V10, ErrorCorrection::L) => 2192,
        (Version::V10, ErrorCorrection::M) => 1728,
        (Version::V10, ErrorCorrection::Q) => 1232,
        (Version::V10, ErrorCorrection::H) => 976,
        // Version 11
        (Version::V11, ErrorCorrection::L) => 2592,
        (Version::V11, ErrorCorrection::M) => 2032,
        (Version::V11, ErrorCorrection::Q) => 1440,
        (Version::V11, ErrorCorrection::H) => 1120,
        // Version 12
        (Version::V12, ErrorCorrection::L) => 2960,
        (Version::V12, ErrorCorrection::M) => 2320,
        (Version::V12, ErrorCorrection::Q) => 1648,
        (Version::V12, ErrorCorrection::H) => 1264,
        // Version 13
        (Version::V13, ErrorCorrection::L) => 3424,
        (Version::V13, ErrorCorrection::M) => 2672,
        (Version::V13, ErrorCorrection::Q) => 1952,
        (Version::V13, ErrorCorrection::H) => 1440,
        // Version 14
        (Version::V14, ErrorCorrection::L) => 3688,
        (Version::V14, ErrorCorrection::M) => 2920,
        (Version::V14, ErrorCorrection::Q) => 2088,
        (Version::V14, ErrorCorrection::H) => 1576,
        // Version 15
        (Version::V15, ErrorCorrection::L) => 4184,
        (Version::V15, ErrorCorrection::M) => 3320,
        (Version::V15, ErrorCorrection::Q) => 2360,
        (Version::V15, ErrorCorrection::H) => 1784,
        // Version 16
        (Version::V16, ErrorCorrection::L) => 4712,
        (Version::V16, ErrorCorrection::M) => 3624,
        (Version::V16, ErrorCorrection::Q) => 2600,
        (Version::V16, ErrorCorrection::H) => 2024,
        // Version 17
        (Version::V17, ErrorCorrection::L) => 5176,
        (Version::V17, ErrorCorrection::M) => 4056,
        (Version::V17, ErrorCorrection::Q) => 2936,
        (Version::V17, ErrorCorrection::H) => 2264,
        // Version 18
        (Version::V18, ErrorCorrection::L) => 5768,
        (Version::V18, ErrorCorrection::M) => 4504,
        (Version::V18, ErrorCorrection::Q) => 3176,
        (Version::V18, ErrorCorrection::H) => 2504,
        // Version 19
        (Version::V19, ErrorCorrection::L) => 6360,
        (Version::V19, ErrorCorrection::M) => 5016,
        (Version::V19, ErrorCorrection::Q) => 3560,
        (Version::V19, ErrorCorrection::H) => 2728,
        // Version 20
        (Version::V20, ErrorCorrection::L) => 6888,
        (Version::V20, ErrorCorrection::M) => 5352,
        (Version::V20, ErrorCorrection::Q) => 3880,
        (Version::V20, ErrorCorrection::H) => 3080,
        // Version 21
        (Version::V21, ErrorCorrection::L) => 7456,
        (Version::V21, ErrorCorrection::M) => 5712,
        (Version::V21, ErrorCorrection::Q) => 4096,
        (Version::V21, ErrorCorrection::H) => 3248,
        // Version 22
        (Version::V22, ErrorCorrection::L) => 8048,
        (Version::V22, ErrorCorrection::M) => 6256,
        (Version::V22, ErrorCorrection::Q) => 4544,
        (Version::V22, ErrorCorrection::H) => 3536,
        // Version 23
        (Version::V23, ErrorCorrection::L) => 8752,
        (Version::V23, ErrorCorrection::M) => 6880,
        (Version::V23, ErrorCorrection::Q) => 4912,
        (Version::V23, ErrorCorrection::H) => 3712,
        // Version 24
        (Version::V24, ErrorCorrection::L) => 9392,
        (Version::V24, ErrorCorrection::M) => 7312,
        (Version::V24, ErrorCorrection::Q) => 5312,
        (Version::V24, ErrorCorrection::H) => 4112,
        // Version 25
        (Version::V25, ErrorCorrection::L) => 10208,
        (Version::V25, ErrorCorrection::M) => 8000,
        (Version::V25, ErrorCorrection::Q) => 5744,
        (Version::V25, ErrorCorrection::H) => 4304,
        // Version 26
        (Version::V26, ErrorCorrection::L) => 10960,
        (Version::V26, ErrorCorrection::M) => 8496,
        (Version::V26, ErrorCorrection::Q) => 6032,
        (Version::V26, ErrorCorrection::H) => 4768,
        // Version 27
        (Version::V27, ErrorCorrection::L) => 11744,
        (Version::V27, ErrorCorrection::M) => 9024,
        (Version::V27, ErrorCorrection::Q) => 6464,
        (Version::V27, ErrorCorrection::H) => 5024,
        // Version 28
        (Version::V28, ErrorCorrection::L) => 12248,
        (Version::V28, ErrorCorrection::M) => 9544,
        (Version::V28, ErrorCorrection::Q) => 6968,
        (Version::V28, ErrorCorrection::H) => 5288,
        // Version 29
        (Version::V29, ErrorCorrection::L) => 13048,
        (Version::V29, ErrorCorrection::M) => 10136,
        (Version::V29, ErrorCorrection::Q) => 7288,
        (Version::V29, ErrorCorrection::H) => 5608,
        // Version 30
        (Version::V30, ErrorCorrection::L) => 13880,
        (Version::V30, ErrorCorrection::M) => 10984,
        (Version::V30, ErrorCorrection::Q) => 7880,
        (Version::V30, ErrorCorrection::H) => 5960,
        // Version 31
        (Version::V31, ErrorCorrection::L) => 14744,
        (Version::V31, ErrorCorrection::M) => 11640,
        (Version::V31, ErrorCorrection::Q) => 8264,
        (Version::V31, ErrorCorrection::H) => 6344,
        // Version 32
        (Version::V32, ErrorCorrection::L) => 15640,
        (Version::V32, ErrorCorrection::M) => 12328,
        (Version::V32, ErrorCorrection::Q) => 8920,
        (Version::V32, ErrorCorrection::H) => 6760,
        // Version 33
        (Version::V33, ErrorCorrection::L) => 16568,
        (Version::V33, ErrorCorrection::M) => 13048,
        (Version::V33, ErrorCorrection::Q) => 9368,
        (Version::V33, ErrorCorrection::H) => 7208,
        // Version 34
        (Version::V34, ErrorCorrection::L) => 17528,
        (Version::V34, ErrorCorrection::M) => 13800,
        (Version::V34, ErrorCorrection::Q) => 9848,
        (Version::V34, ErrorCorrection::H) => 7688,
        // Version 35
        (Version::V35, ErrorCorrection::L) => 18448,
        (Version::V35, ErrorCorrection::M) => 14496,
        (Version::V35, ErrorCorrection::Q) => 10288,
        (Version::V35, ErrorCorrection::H) => 7888,
        // Version 36
        (Version::V36, ErrorCorrection::L) => 19472,
        (Version::V36, ErrorCorrection::M) => 15312,
        (Version::V36, ErrorCorrection::Q) => 10832,
        (Version::V36, ErrorCorrection::H) => 8432,
        // Version 37
        (Version::V37, ErrorCorrection::L) => 20528,
        (Version::V37, ErrorCorrection::M) => 15936,
        (Version::V37, ErrorCorrection::Q) => 11408,
        (Version::V37, ErrorCorrection::H) => 8768,
        // Version 38
        (Version::V38, ErrorCorrection::L) => 21616,
        (Version::V38, ErrorCorrection::M) => 16816,
        (Version::V38, ErrorCorrection::Q) => 12016,
        (Version::V38, ErrorCorrection::H) => 9136,
        // Version 39
        (Version::V39, ErrorCorrection::L) => 22496,
        (Version::V39, ErrorCorrection::M) => 17728,
        (Version::V39, ErrorCorrection::Q) => 12656,
        (Version::V39, ErrorCorrection::H) => 9776,
        // Version 40
        (Version::V40, ErrorCorrection::L) => 23648,
        (Version::V40, ErrorCorrection::M) => 18672,
        (Version::V40, ErrorCorrection::Q) => 13328,
        (Version::V40, ErrorCorrection::H) => 10208,
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
