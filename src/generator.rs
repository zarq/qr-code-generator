use crate::types::{Version, ErrorCorrection, MaskPattern, DataMode, QrConfig};
use crate::mask::apply_mask;
use crate::encoding::{encode_data, EncodedData};
use crate::alignment::{is_alignment_pattern, get_alignment_positions};
use crate::capacity::get_unencoded_capacity_in_bytes;
use crate::pixel_mapping::get_data_ecc_positions;

pub fn generate_qr_matrix(data: &str, config: &QrConfig) -> Vec<Vec<u8>> {
    let version = calculate_version(data, config.error_correction, config.data_mode);
    let size = 21 + (version as usize - 1) * 4;
    let mut matrix = vec![vec![0u8; size]; size];

    let encoded = encode_data(data, version, config.error_correction, config.data_mode);
    
    if config.verbose {
        println!("Original data (hex): {}", hex::encode(data.as_bytes()).chars().collect::<Vec<_>>().chunks(2).map(|chunk| chunk.iter().collect::<String>()).collect::<Vec<_>>().join(" "));
        println!("Serialized data (hex): {}", hex::encode(&bits_to_bytes(&encoded.data_bits)).chars().collect::<Vec<_>>().chunks(2).map(|chunk| chunk.iter().collect::<String>()).collect::<Vec<_>>().join(" "));
        println!("ECC data (hex): {}", hex::encode(&bits_to_bytes(&encoded.ecc_bits)).chars().collect::<Vec<_>>().chunks(2).map(|chunk| chunk.iter().collect::<String>()).collect::<Vec<_>>().join(" "));
        
        let mut full_bits = encoded.data_bits.clone();
        full_bits.extend(&encoded.ecc_bits);
        let bit_string: String = full_bits.iter().map(|&b| if b == 1 { '1' } else { '0' }).collect();
        println!("Full bit string (unmasked): {}", bit_string);
    }
    
    place_data_bits(&mut matrix, &encoded, version);

    if !config.skip_mask {
        apply_mask(&mut matrix, config.mask_pattern);
    }

    // Add finder patterns
    add_position_pattern(&mut matrix, 0, 0);
    add_position_pattern(&mut matrix, size - 7, 0);
    add_position_pattern(&mut matrix, 0, size - 7);

    // Add separators (white borders around finder patterns)
    add_timing_patterns(&mut matrix, size);
    add_alignment_patterns(&mut matrix, version);
    add_dark_module(&mut matrix, version);

    if version >= Version::V7 {
        add_version_info(&mut matrix, version);
    }

    add_format_info(&mut matrix, config.error_correction, config.mask_pattern);

    matrix
}

pub fn calculate_version(data: &str, error_correction: ErrorCorrection, data_mode: DataMode) -> Version {
    for version in 1..=40 {
        let version_enum = match version {
            1 => Version::V1, 2 => Version::V2, 3 => Version::V3, 4 => Version::V4, 5 => Version::V5,
            6 => Version::V6, 7 => Version::V7, 8 => Version::V8, 9 => Version::V9, 10 => Version::V10,
            11 => Version::V11, 12 => Version::V12, 13 => Version::V13, 14 => Version::V14, 15 => Version::V15,
            16 => Version::V16, 17 => Version::V17, 18 => Version::V18, 19 => Version::V19, 20 => Version::V20,
            21 => Version::V21, 22 => Version::V22, 23 => Version::V23, 24 => Version::V24, 25 => Version::V25,
            26 => Version::V26, 27 => Version::V27, 28 => Version::V28, 29 => Version::V29, 30 => Version::V30,
            31 => Version::V31, 32 => Version::V32, 33 => Version::V33, 34 => Version::V34, 35 => Version::V35,
            36 => Version::V36, 37 => Version::V37, 38 => Version::V38, 39 => Version::V39, 40 => Version::V40,
            _ => continue,
        };
        
        let capacity = get_unencoded_capacity_in_bytes(version_enum, error_correction, data_mode);
        if data.len() <= capacity {
            return version_enum;
        }
    }
    Version::V40
}

fn add_position_pattern(matrix: &mut Vec<Vec<u8>>, x: usize, y: usize) {
    let size = matrix.len();
    
    // White border (9x9)
    for dy in 0..9 {
        for dx in 0..9 {
            let row = y + dy;
            let col = x + dx;
            if row > 0 && col > 0 && row < size + 1 && col < size + 1 {
                matrix[row - 1][col - 1] = 0;
            }
        }
    }
    
    // Black outer ring (7x7)
    for dy in 1..8 {
        for dx in 1..8 {
            let row = y + dy;
            let col = x + dx;
            if row < size + 1 && col < size + 1 {
                matrix[row - 1][col - 1] = 1;
            }
        }
    }
    
    // White inner area (5x5)
    for dy in 2..7 {
        for dx in 2..7 {
            let row = y + dy;
            let col = x + dx;
            if row < size && col < size {
                matrix[row - 1][col - 1] = 0;
            }
        }
    }
    
    // Black center square (3x3)
    for dy in 3..6 {
        for dx in 3..6 {
            let row = y + dy;
            let col = x + dx;
            if row < size && col < size {
                matrix[row - 1][col - 1] = 1;
            }
        }
    }
}

fn add_alignment_pattern(matrix: &mut Vec<Vec<u8>>, x: usize, y: usize) {
    for dy in 0..5 {
        for dx in 0..5 {
            matrix[y + dy][x + dx] = if (dy == 0 || dy == 4 || dx == 0 || dx == 4) || (dy == 2 && dx == 2) { 1 } else { 0 };
        }
    }
}

fn add_timing_patterns(matrix: &mut Vec<Vec<u8>>, size: usize) {
    for i in 8..size-8 {
        matrix[6][i] = ((i + 1) % 2) as u8;
        matrix[i][6] = ((i + 1) % 2) as u8;
    }
}

fn get_format_info(error_correction: ErrorCorrection, mask_pattern: MaskPattern) -> u16 {
    match (error_correction, mask_pattern) {
        (ErrorCorrection::L, MaskPattern::Pattern0) => 0b111011111000100,
        (ErrorCorrection::L, MaskPattern::Pattern1) => 0b111001011110011,
        (ErrorCorrection::L, MaskPattern::Pattern2) => 0b111110110101010,
        (ErrorCorrection::L, MaskPattern::Pattern3) => 0b111100010011101,
        (ErrorCorrection::L, MaskPattern::Pattern4) => 0b110011000101111,
        (ErrorCorrection::L, MaskPattern::Pattern5) => 0b110001100011000,
        (ErrorCorrection::L, MaskPattern::Pattern6) => 0b110110001000001,
        (ErrorCorrection::L, MaskPattern::Pattern7) => 0b110100101110110,
        (ErrorCorrection::M, MaskPattern::Pattern0) => 0b101010000010010,
        (ErrorCorrection::M, MaskPattern::Pattern1) => 0b101000100100101,
        (ErrorCorrection::M, MaskPattern::Pattern2) => 0b101111001111100,
        (ErrorCorrection::M, MaskPattern::Pattern3) => 0b101101101001011,
        (ErrorCorrection::M, MaskPattern::Pattern4) => 0b100010111111001,
        (ErrorCorrection::M, MaskPattern::Pattern5) => 0b100000011001110,
        (ErrorCorrection::M, MaskPattern::Pattern6) => 0b100111110010111,
        (ErrorCorrection::M, MaskPattern::Pattern7) => 0b100101010100000,
        (ErrorCorrection::Q, MaskPattern::Pattern0) => 0b011010101011111,
        (ErrorCorrection::Q, MaskPattern::Pattern1) => 0b011000001101000,
        (ErrorCorrection::Q, MaskPattern::Pattern2) => 0b011111100110001,
        (ErrorCorrection::Q, MaskPattern::Pattern3) => 0b011101000000110,
        (ErrorCorrection::Q, MaskPattern::Pattern4) => 0b010010010110100,
        (ErrorCorrection::Q, MaskPattern::Pattern5) => 0b010000110000011,
        (ErrorCorrection::Q, MaskPattern::Pattern6) => 0b010111011011010,
        (ErrorCorrection::Q, MaskPattern::Pattern7) => 0b010101111101101,
        (ErrorCorrection::H, MaskPattern::Pattern0) => 0b001011010001001,
        (ErrorCorrection::H, MaskPattern::Pattern1) => 0b001001110111110,
        (ErrorCorrection::H, MaskPattern::Pattern2) => 0b001110011100111,
        (ErrorCorrection::H, MaskPattern::Pattern3) => 0b001100111010000,
        (ErrorCorrection::H, MaskPattern::Pattern4) => 0b000011101100010,
        (ErrorCorrection::H, MaskPattern::Pattern5) => 0b000001001010101,
        (ErrorCorrection::H, MaskPattern::Pattern6) => 0b000110100001100,
        (ErrorCorrection::H, MaskPattern::Pattern7) => 0b000100000111011,
    }
}

fn add_format_info(matrix: &mut Vec<Vec<u8>>, error_correction: ErrorCorrection, mask_pattern: MaskPattern) {
    let format_info = get_format_info(error_correction, mask_pattern);
    let size = matrix.len();
    
    // Place format info bits around top-left finder pattern
    // Bits 0-5: horizontal, left to right
    for i in 0..6 {
        matrix[8][i] = ((format_info >> (14-i)) & 1) as u8;
    }
    // Skip timing pattern at (8,6)
    // Bit 6: at (8,7)
    matrix[8][7] = ((format_info >> 8) & 1) as u8;
    // Bit 7: at (8,8) 
    matrix[8][8] = ((format_info >> 7) & 1) as u8;
    // Bit 8: at (7,8)
    matrix[7][8] = ((format_info >> 6) & 1) as u8;
    // Bits 9-14: vertical, bottom to top
    for i in 0..6 {
        matrix[5-i][8] = ((format_info >> (5-i)) & 1) as u8;
    }
    
    // Place format info bits around bottom-left and top-right finder patterns
    // Bottom-left vertical (bits 14-8, reversed)
    for i in 0..7 {
        matrix[size-1-i][8] = ((format_info >> (14-i)) & 1) as u8;
    }
    // Top-right horizontal (bits 7-0, reversed)  
    for i in 0..8 {
        matrix[8][size-8+i] = ((format_info >> (7-i)) & 1) as u8;
    }
}

fn place_data_bits(matrix: &mut Vec<Vec<u8>>, encoded: &EncodedData, version: Version) {
    // Convert bits to bytes and combine data + ECC
    let data_bytes = bits_to_bytes(&encoded.data_bits);
    let ecc_bytes = bits_to_bytes(&encoded.ecc_bits);
    
    let mut all_bytes = data_bytes;
    all_bytes.extend(ecc_bytes);
    
    // Convert bytes back to bits for placement
    let mut all_bits = Vec::new();
    for byte in all_bytes {
        for i in (0..8).rev() {
            all_bits.push((byte >> i) & 1);
        }
    }
    
    // Get the correct data placement positions
    let positions = get_data_ecc_positions(version);
    
    // Place bits at the designated positions
    for (bit_index, &(row, col)) in positions.iter().enumerate() {
        if bit_index < all_bits.len() {
            matrix[row][col] = all_bits[bit_index];
        }
    }
}

fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for chunk in bits.chunks(8) {
        let mut byte = 0u8;
        for (i, &bit) in chunk.iter().enumerate() {
            byte |= bit << (7 - i);
        }
        bytes.push(byte);
    }
    bytes
}

fn is_function_module(x: usize, y: usize, size: usize, version: Version) -> bool {
    // Finder patterns and separators
    if (x < 9 && y < 9) || (x >= size - 8 && y < 9) || (x < 9 && y >= size - 8) {
        return true;
    }
    
    // Timing patterns
    if x == 6 || y == 6 {
        return true;
    }
    
    // Dark module
    if x == 8 && y == 4 * version as usize + 9 {
        return true;
    }
    
    // Version information
    if version >= Version::V7 {
        if (x < 6 && y >= size - 11) || (y < 6 && x >= size - 11) {
            return true;
        }
    }
    
    // Format information
    if (x == 8 && (y < 9 || y >= size - 8)) || (y == 8 && (x < 9 || x >= size - 7)) {
        return true;
    }
    
    // Alignment patterns
    is_alignment_pattern(x, y, version)
}

fn get_version_info(version: Version) -> Option<u32> {
    match version {
        Version::V7 => Some(0x07C94),
        Version::V8 => Some(0x085BC),
        Version::V9 => Some(0x09A99),
        Version::V10 => Some(0x0A4D3),
        Version::V11 => Some(0x0BBF6),
        Version::V12 => Some(0x0C762),
        Version::V13 => Some(0x0D847),
        Version::V14 => Some(0x0E60D),
        Version::V15 => Some(0x0F928),
        Version::V16 => Some(0x10B78),
        Version::V17 => Some(0x1145D),
        Version::V18 => Some(0x12A17),
        Version::V19 => Some(0x13532),
        Version::V20 => Some(0x149A6),
        _ => None,
    }
}

fn add_version_info(matrix: &mut Vec<Vec<u8>>, version: Version) {
    if let Some(version_info) = get_version_info(version) {
        let size = matrix.len();
        
        for i in 0..18 {
            let bit = ((version_info >> i) & 1) as u8;
            matrix[i / 3][size - 11 + i % 3] = bit;
            matrix[size - 11 + i % 3][i / 3] = bit;
        }
    }
}

fn add_alignment_patterns(matrix: &mut Vec<Vec<u8>>, version: Version) {
    let positions = get_alignment_positions(version);
    
    for &y in &positions {
        for &x in &positions {
            if !((x < 9 && y < 9) || (x >= matrix.len() - 8 && y < 9) || (x < 9 && y >= matrix.len() - 8)) {
                add_alignment_pattern(matrix, x - 2, y - 2);
            }
        }
    }
}

fn add_dark_module(matrix: &mut Vec<Vec<u8>>, version: Version) {
    matrix[4 * version as usize + 9][8] = 1;
}
