use crate::types::{Version, ErrorCorrection, MaskPattern, DataMode, QrConfig};
use crate::mask::apply_mask;
use crate::encoding::{encode_data, EncodedData};
use crate::alignment::{is_alignment_pattern, get_alignment_positions};
use crate::capacity::get_unencoded_capacity_in_bytes;

pub fn generate_qr_matrix(data: &str, config: &QrConfig) -> Vec<Vec<u8>> {
    let version = calculate_version(data, config.error_correction, config.data_mode);
    let size = 21 + (version as usize - 1) * 4;
    let mut matrix = vec![vec![0u8; size]; size];

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

    let encoded = encode_data(data, version, config.error_correction, config.data_mode);
    place_data_bits(&mut matrix, &encoded, version);

    if !config.skip_mask {
        apply_mask(&mut matrix, config.mask_pattern);
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
    let ec_bits = match error_correction {
        ErrorCorrection::L => 0b01,
        ErrorCorrection::M => 0b00,
        ErrorCorrection::Q => 0b11,
        ErrorCorrection::H => 0b10,
    };
    
    let mask_bits = match mask_pattern {
        MaskPattern::Pattern0 => 0b000,
        MaskPattern::Pattern1 => 0b001,
        MaskPattern::Pattern2 => 0b010,
        MaskPattern::Pattern3 => 0b011,
        MaskPattern::Pattern4 => 0b100,
        MaskPattern::Pattern5 => 0b101,
        MaskPattern::Pattern6 => 0b110,
        MaskPattern::Pattern7 => 0b111,
    };
    
    let data = (ec_bits << 3) | mask_bits;
    let mut format_info = (data as u16) << 10;
    
    // BCH(15,5) encoding with generator polynomial x^10 + x^8 + x^5 + x^4 + x^2 + x + 1
    let generator = 0b10100110111;
    let mut remainder = format_info;
    
    for _ in 0..5 {
        if remainder & 0x4000 != 0 {
            remainder = (remainder << 1) ^ generator;
        } else {
            remainder <<= 1;
        }
    }
    
    format_info |= remainder & 0x3FF;
    format_info ^ 0x5412 // Apply mask
}

fn add_format_info(matrix: &mut Vec<Vec<u8>>, error_correction: ErrorCorrection, mask_pattern: MaskPattern) {
    let format_info = get_format_info(error_correction, mask_pattern);
    let size = matrix.len();
    
    // Place format info bits around top-left finder pattern
    for i in 0..6 {
        matrix[8][i] = ((format_info >> i) & 1) as u8;
    }
    matrix[8][7] = ((format_info >> 6) & 1) as u8;
    matrix[8][8] = ((format_info >> 7) & 1) as u8;
    matrix[7][8] = ((format_info >> 8) & 1) as u8;
    for i in 0..6 {
        matrix[5-i][8] = ((format_info >> (9+i)) & 1) as u8;
    }
    
    // Place format info bits around other finder patterns
    for i in 0..8 {
        matrix[size-1-i][8] = ((format_info >> i) & 1) as u8;
    }
    for i in 0..7 {
        matrix[8][size-7+i] = ((format_info >> (8+i)) & 1) as u8;
    }
}

fn place_data_bits(matrix: &mut Vec<Vec<u8>>, encoded: &EncodedData, version: Version) {
    let size = matrix.len();
    let (data_blocks, ecc_blocks) = get_block_structure(&encoded.data_bits, &encoded.ecc_bits);
    
    let mut all_bits = Vec::new();
    let max_data_blocks = data_blocks.len();
    let max_ecc_blocks = ecc_blocks.len();
    let max_data_len = data_blocks.iter().map(|b| b.len()).max().unwrap_or(0);
    let max_ecc_len = ecc_blocks.iter().map(|b| b.len()).max().unwrap_or(0);
    
    // Interleave data blocks
    for i in 0..max_data_len {
        for j in 0..max_data_blocks {
            if i < data_blocks[j].len() {
                all_bits.push(data_blocks[j][i]);
            }
        }
    }
    
    // Interleave ECC blocks
    for i in 0..max_ecc_len {
        for j in 0..max_ecc_blocks {
            if i < ecc_blocks[j].len() {
                all_bits.push(ecc_blocks[j][i]);
            }
        }
    }
    
    let mut bit_index = 0;
    let mut up = true;
    let mut col = size - 1;
    
    while col > 0 {
        if col == 6 { col -= 1; }
        
        for _ in 0..2 {
            let mut row = if up { size - 1 } else { 0 };
            
            loop {
                if !is_function_module(col, row, size, version) {
                    if bit_index < all_bits.len() {
                        matrix[row][col] = all_bits[bit_index];
                        bit_index += 1;
                    }
                }
                
                if up {
                    if row == 0 { break; }
                    row -= 1;
                } else {
                    row += 1;
                    if row >= size { break; }
                }
            }
            
            if col == 0 { break; }
            col -= 1;
        }
        
        up = !up;
        if col == 0 { break; }
        col -= 1;
    }
}

fn get_block_structure(data_bits: &[u8], ecc_bits: &[u8]) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let data_bytes = bits_to_bytes(data_bits);
    let ecc_bytes = bits_to_bytes(ecc_bits);
    
    let data_blocks = vec![data_bytes];
    let ecc_blocks = vec![ecc_bytes];
    
    (data_blocks, ecc_blocks)
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

fn add_dark_module(matrix: &mut Vec<Vec<u8>>, _version: Version) {
    let size = matrix.len();
    matrix[4 * _version as usize + 9][8] = 1;
}
