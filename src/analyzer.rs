use image;
use std::env;
use serde::Serialize;

mod types;
mod mask;
use types::{Version, ErrorCorrection, MaskPattern, DataMode};

#[derive(Debug, Serialize)]
struct BorderCheck {
    has_border: bool,
    border_width: usize,
    valid: bool,
}

#[derive(Debug, Serialize)]
struct QrAnalysis {
    status: String,
    version_from_size: Option<Version>,
    version_from_format: Option<Version>,
    versions_match: bool,
    size: usize,
    error_correction: Option<ErrorCorrection>,
    data_mode: Option<DataMode>,
    mask_pattern: Option<MaskPattern>,
    raw_data: Option<String>,
    decoded_text: Option<String>,
    data_analysis: DataAnalysis,
    format_info: FormatInfo,
    finder_patterns: Vec<FinderPattern>,
    timing_patterns: TimingPatterns,
    dark_module: DarkModule,
    alignment_patterns: Vec<AlignmentPattern>,
    border_check: BorderCheck,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct FormatInfo {
    raw_bits_copy1: Option<String>,
    raw_bits_copy2: Option<String>,
    copies_match: bool,
    error_correction: Option<ErrorCorrection>,
    mask_pattern: Option<MaskPattern>,
    version: Option<Version>,
}

#[derive(Debug, Serialize)]
struct FinderPattern {
    position: String,
    valid: bool,
}

#[derive(Debug, Serialize)]
struct TimingPatterns {
    valid: bool,
}

#[derive(Debug, Serialize)]
struct DarkModule {
    present: bool,
    position: (usize, usize),
}

#[derive(Debug, Serialize)]
struct DataAnalysis {
    full_bit_string: Option<String>,
    unmasked_bit_string: Option<String>,
    encoding_info: Option<String>,
    encoding_mode: Option<String>,
    data_length: Option<usize>,
    extracted_data: Option<String>,
    ecc_bits: Option<String>,
    data_ecc_valid: bool,
    data_size: Option<usize>,
    bit_string_size: Option<usize>,
    padding_bits: Option<usize>,
}

#[derive(Debug, Serialize)]
struct AlignmentPattern {
    x: usize,
    y: usize,
    valid: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <qr-code.png>", args[0]);
        std::process::exit(1);
    }
    
    let filename = &args[1];
    let analysis = analyze_qr_code(filename)?;
    
    println!("{}", serde_json::to_string_pretty(&analysis)?);
    Ok(())
}

fn analyze_qr_code(filename: &str) -> Result<QrAnalysis, Box<dyn std::error::Error>> {
    let img = image::open(filename)?;
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    
    if width != height {
        return Err("QR code must be square".into());
    }
    
    let size = width as usize;
    
    // Check for 2-pixel white border
    let border_check = check_border(&rgb_img, size);
    let inner_size = if border_check.valid { size - 4 } else { size };
    let offset = if border_check.valid { 2 } else { 0 };
    
    let mut matrix = vec![vec![0u8; inner_size]; inner_size];
    
    // Convert image to binary matrix (skip border if present)
    for y in 0..inner_size {
        for x in 0..inner_size {
            let pixel = rgb_img.get_pixel((x + offset) as u32, (y + offset) as u32);
            matrix[y][x] = if pixel[0] < 128 { 1 } else { 0 };
        }
    }
    
    let mut analysis = QrAnalysis {
        status: "success".to_string(),
        version_from_size: None,
        version_from_format: None,
        versions_match: false,
        size: inner_size,
        error_correction: None,
        data_mode: None,
        mask_pattern: None,
        raw_data: None,
        decoded_text: None,
        format_info: FormatInfo {
            raw_bits_copy1: None,
            raw_bits_copy2: None,
            copies_match: false,
            error_correction: None,
            mask_pattern: None,
            version: None,
        },
        data_analysis: DataAnalysis {
            full_bit_string: None,
            unmasked_bit_string: None,
            encoding_info: None,
            encoding_mode: None,
            data_length: None,
            extracted_data: None,
            ecc_bits: None,
            data_ecc_valid: false,
            data_size: None,
            bit_string_size: None,
            padding_bits: None,
        },
        finder_patterns: Vec::new(),
        timing_patterns: TimingPatterns { valid: false },
        dark_module: DarkModule { present: false, position: (0, 0) },
        alignment_patterns: Vec::new(),
        border_check,
        errors: Vec::new(),
        warnings: Vec::new(),
    };
    
    // Determine version from size
    analysis.version_from_size = match inner_size {
        21 => Some(Version::V1),
        25 => Some(Version::V2),
        29 => Some(Version::V3),
        33 => Some(Version::V4),
        _ => {
            analysis.errors.push(format!("Unsupported QR code size: {}x{}", inner_size, inner_size));
            None
        }
    };
    
    // Analyze finder patterns
    analysis.finder_patterns = analyze_finder_patterns(&matrix);
    
    // Analyze timing patterns
    analysis.timing_patterns = analyze_timing_patterns(&matrix);
    
    // Analyze dark module
    analysis.dark_module = analyze_dark_module(&matrix);
    
    // Analyze format information
    if let Some(mut format_info) = analyze_format_info(&matrix) {
        // For V1-V6, version is implicit from size, so use size-based version
        format_info.version = analysis.version_from_size;
        analysis.format_info = format_info;
        analysis.error_correction = analysis.format_info.error_correction;
        analysis.mask_pattern = analysis.format_info.mask_pattern;
        analysis.version_from_format = analysis.format_info.version;
    }
    
    // Check if versions match
    analysis.versions_match = analysis.version_from_size == analysis.version_from_format;
    
    // Analyze alignment patterns (for V2+)
    if let Some(version) = analysis.version_from_size {
        if matches!(version, Version::V2 | Version::V3 | Version::V4) {
            analysis.alignment_patterns = analyze_alignment_patterns(&matrix, version);
        }
    }
    
    // Try to decode data
    if let Some(mask) = analysis.mask_pattern {
        analysis.data_analysis = decode_data_comprehensive(&matrix, mask, analysis.version_from_size, analysis.error_correction);
        analysis.data_mode = analysis.data_analysis.encoding_info.as_ref().and_then(|info| {
            match info.chars().take(4).collect::<String>().as_str() {
                "0001" => Some(DataMode::Numeric),
                "0010" => Some(DataMode::Alphanumeric),
                "0100" => Some(DataMode::Byte),
                _ => None,
            }
        });
        analysis.decoded_text = analysis.data_analysis.extracted_data.clone();
        analysis.raw_data = analysis.data_analysis.extracted_data.clone();
    }
    
    // Set status based on errors
    if !analysis.errors.is_empty() {
        analysis.status = "failed".to_string();
    } else if !analysis.warnings.is_empty() {
        analysis.status = "partial".to_string();
    }
    
    Ok(analysis)
}

fn check_border(img: &image::RgbImage, size: usize) -> BorderCheck {
    let mut has_border = true;
    let border_width = 2;
    
    // Check top and bottom borders
    for x in 0..size {
        for y in 0..border_width {
            let top_pixel = img.get_pixel(x as u32, y as u32);
            let bottom_pixel = img.get_pixel(x as u32, (size - 1 - y) as u32);
            if top_pixel[0] < 200 || bottom_pixel[0] < 200 {
                has_border = false;
                break;
            }
        }
        if !has_border { break; }
    }
    
    // Check left and right borders
    if has_border {
        for y in 0..size {
            for x in 0..border_width {
                let left_pixel = img.get_pixel(x as u32, y as u32);
                let right_pixel = img.get_pixel((size - 1 - x) as u32, y as u32);
                if left_pixel[0] < 200 || right_pixel[0] < 200 {
                    has_border = false;
                    break;
                }
            }
            if !has_border { break; }
        }
    }
    
    BorderCheck {
        has_border,
        border_width: if has_border { border_width } else { 0 },
        valid: has_border,
    }
}

fn analyze_finder_patterns(matrix: &[Vec<u8>]) -> Vec<FinderPattern> {
    let mut patterns = Vec::new();
    let size = matrix.len();
    
    // Check top-left
    patterns.push(FinderPattern {
        position: "top-left".to_string(),
        valid: check_finder_pattern(matrix, 0, 0),
    });
    
    // Check top-right
    patterns.push(FinderPattern {
        position: "top-right".to_string(),
        valid: check_finder_pattern(matrix, size - 7, 0),
    });
    
    // Check bottom-left
    patterns.push(FinderPattern {
        position: "bottom-left".to_string(),
        valid: check_finder_pattern(matrix, 0, size - 7),
    });
    
    patterns
}

fn check_finder_pattern(matrix: &[Vec<u8>], start_x: usize, start_y: usize) -> bool {
    let expected = [
        [1,1,1,1,1,1,1],
        [1,0,0,0,0,0,1],
        [1,0,1,1,1,0,1],
        [1,0,1,1,1,0,1],
        [1,0,1,1,1,0,1],
        [1,0,0,0,0,0,1],
        [1,1,1,1,1,1,1],
    ];
    
    for y in 0..7 {
        for x in 0..7 {
            if matrix[start_y + y][start_x + x] != expected[y][x] {
                return false;
            }
        }
    }
    true
}

fn analyze_timing_patterns(matrix: &[Vec<u8>]) -> TimingPatterns {
    let size = matrix.len();
    let mut valid = true;
    
    // Check horizontal timing pattern
    for i in 8..(size - 8) {
        let expected = ((i + 1) % 2) as u8;
        if matrix[6][i] != expected {
            valid = false;
            break;
        }
    }
    
    // Check vertical timing pattern
    if valid {
        for i in 8..(size - 8) {
            let expected = ((i + 1) % 2) as u8;
            if matrix[i][6] != expected {
                valid = false;
                break;
            }
        }
    }
    
    TimingPatterns { valid }
}

fn analyze_dark_module(matrix: &[Vec<u8>]) -> DarkModule {
    let size = matrix.len();
    let row = size - 8;
    let col = 8;
    let present = matrix[row][col] == 1;
    
    DarkModule {
        present,
        position: (row, col),
    }
}

fn analyze_format_info(matrix: &[Vec<u8>]) -> Option<FormatInfo> {
    let size = matrix.len();
    
    // Read format info copy 1 (around top-left finder pattern)
    let mut bits1 = Vec::new();
    // Horizontal part: positions (8,0) to (8,5)
    for i in 0..6 {
        bits1.push(matrix[8][i]);
    }
    // Skip timing pattern at (8,6)
    // Position (8,7)
    bits1.push(matrix[8][7]);
    // Position (8,8) 
    bits1.push(matrix[8][8]);
    // Vertical part: positions (7,8) down to (0,8)
    bits1.push(matrix[7][8]);
    for i in (0..6).rev() {
        bits1.push(matrix[i][8]);
    }
    
    // Read format info copy 2 (split between top-right and bottom-left)
    let mut bits2 = Vec::new();
    // Bottom-left part first: positions (size-1, 8) to (size-7, 8) - reading bottom to top, skip dark module
    for i in (size-7..size).rev() {
        if i != size - 8 { // Skip dark module position
            bits2.push(matrix[i][8]);
        }
    }
    // Add the shared bit at (8,8)
    bits2.push(matrix[8][8]);
    // Top-right part: positions (8, size-7) to (8, size-1) - reading left to right
    for i in size-7..size {
        bits2.push(matrix[8][i]);
    }
    
    let raw_bits1 = bits1.iter().map(|&b| if b == 1 { '1' } else { '0' }).collect::<String>();
    let raw_bits2 = bits2.iter().map(|&b| if b == 1 { '1' } else { '0' }).collect::<String>();
    let copies_match = raw_bits1 == raw_bits2;
    
    // Decode format info from copy 1
    let format_value = bits_to_u16(&bits1);
    let (ecc, mask, _) = decode_format_info(format_value);
    
    Some(FormatInfo {
        raw_bits_copy1: Some(raw_bits1),
        raw_bits_copy2: Some(raw_bits2),
        copies_match,
        error_correction: ecc,
        mask_pattern: mask,
        version: None,
    })
}

fn analyze_alignment_patterns(matrix: &[Vec<u8>], version: Version) -> Vec<AlignmentPattern> {
    let mut patterns = Vec::new();
    
    match version {
        Version::V2 => {
            patterns.push(AlignmentPattern {
                x: 16,
                y: 16,
                valid: check_alignment_pattern(matrix, 16, 16),
            });
        }
        Version::V3 => {
            patterns.push(AlignmentPattern {
                x: 20,
                y: 20,
                valid: check_alignment_pattern(matrix, 20, 20),
            });
        }
        Version::V4 => {
            patterns.push(AlignmentPattern {
                x: 24,
                y: 24,
                valid: check_alignment_pattern(matrix, 24, 24),
            });
        }
        _ => {}
    }
    
    patterns
}

fn check_alignment_pattern(matrix: &[Vec<u8>], center_x: usize, center_y: usize) -> bool {
    let expected = [
        [1,1,1,1,1],
        [1,0,0,0,1],
        [1,0,1,0,1],
        [1,0,0,0,1],
        [1,1,1,1,1],
    ];
    
    for y in 0..5 {
        for x in 0..5 {
            let matrix_x = center_x - 2 + x;
            let matrix_y = center_y - 2 + y;
            if matrix[matrix_y][matrix_x] != expected[y][x] {
                return false;
            }
        }
    }
    true
}

fn decode_data_comprehensive(matrix: &[Vec<u8>], mask: MaskPattern, version: Option<Version>, ecc_level: Option<ErrorCorrection>) -> DataAnalysis {
    let size = matrix.len();
    
    // Step 1: Read raw bit string from matrix
    let raw_bits = read_data_bits(matrix, size);
    let raw_bit_string = raw_bits.iter().map(|&b| if b == 1 { '1' } else { '0' }).collect::<String>();
    
    // Step 2: Apply mask to matrix and read unmasked bits
    let mut unmasked_matrix = matrix.to_vec();
    mask::apply_mask(&mut unmasked_matrix, mask);
    let unmasked_bits = read_data_bits(&unmasked_matrix, size);
    let unmasked_bit_string = unmasked_bits.iter().map(|&b| if b == 1 { '1' } else { '0' }).collect::<String>();
    
    if unmasked_bits.len() < 8 {
        return DataAnalysis {
            full_bit_string: Some(raw_bit_string),
            unmasked_bit_string: Some(unmasked_bit_string),
            encoding_info: None,
            encoding_mode: None,
            data_length: None,
            extracted_data: None,
            ecc_bits: None,
            data_ecc_valid: false,
            data_size: None,
            bit_string_size: Some(raw_bits.len()),
            padding_bits: None,
        };
    }
    
    // Step 3: Analyze unmasked data
    let mode_bits = &unmasked_bits[0..4];
    let encoding_info = mode_bits.iter().map(|&b| if b == 1 { '1' } else { '0' }).collect::<String>();
    let encoding_mode = match encoding_info.as_str() {
        "0001" => Some("Numeric".to_string()),
        "0010" => Some("Alphanumeric".to_string()),
        "0100" => Some("Byte".to_string()),
        "1000" => Some("Kanji".to_string()),
        _ => Some("Unknown".to_string()),
    };
    
    let length_bits = match encoding_info.as_str() {
        "0001" => 10, // Numeric mode in V1 uses 10 bits for length
        "0010" => 9,  // Alphanumeric mode in V1 uses 9 bits
        "0100" => 8,  // Byte mode in V1 uses 8 bits
        _ => 8,
    };
    let data_length = if unmasked_bits.len() >= 4 + length_bits {
        Some(bits_to_usize(&unmasked_bits[4..4+length_bits]))
    } else {
        None
    };
    
    let data_start = 4 + length_bits;
    let data_bits_needed = match encoding_info.as_str() {
        "0001" => data_length.map(|len| len * 10 / 3 + if len % 3 > 0 { 1 } else { 0 }).unwrap_or(0),
        "0010" => data_length.map(|len| len * 11 / 2 + if len % 2 > 0 { 1 } else { 0 }).unwrap_or(0),
        "0100" => data_length.map(|len| len * 8).unwrap_or(0),
        _ => 0,
    };
    
    let data_end = std::cmp::min(data_start + data_bits_needed, unmasked_bits.len());
    let extracted_data = if data_end > data_start {
        decode_data_bits(&unmasked_bits[data_start..data_end], &encoding_info)
    } else {
        None
    };
    
    let total_capacity = get_total_capacity(version, ecc_level);
    let data_capacity = get_data_capacity(version, ecc_level);
    
    let ecc_start = data_capacity.unwrap_or(unmasked_bits.len());
    let ecc_bits = if ecc_start < unmasked_bits.len() {
        Some(unmasked_bits[ecc_start..std::cmp::min(ecc_start + (total_capacity.unwrap_or(unmasked_bits.len()) - data_capacity.unwrap_or(0)), unmasked_bits.len())]
            .iter().map(|&b| if b == 1 { '1' } else { '0' }).collect::<String>())
    } else {
        None
    };
    
    let padding_bits = if data_end < ecc_start {
        Some(ecc_start - data_end)
    } else {
        None
    };
    
    let data_ecc_valid = if let (Some(data_cap), Some(total_cap)) = (data_capacity, total_capacity) {
        validate_ecc(&unmasked_bits, data_cap, total_cap - data_cap, ecc_level)
    } else {
        false
    };
    
    DataAnalysis {
        full_bit_string: Some(raw_bit_string),
        unmasked_bit_string: Some(unmasked_bit_string),
        encoding_info: Some(encoding_info),
        encoding_mode,
        data_length,
        extracted_data,
        ecc_bits,
        data_ecc_valid,
        data_size: data_length,
        bit_string_size: Some(raw_bits.len()),
        padding_bits,
    }
}

fn read_data_bits(matrix: &[Vec<u8>], size: usize) -> Vec<u8> {
    let mut bits = Vec::new();
    let mut col = size - 1;
    let mut going_up = true;
    
    while col > 0 {
        if col == 6 { col -= 1; } // Skip timing column
        
        if going_up {
            // Read from bottom to top
            for row in (0..size).rev() {
                // Read right column first, then left column
                for offset in [0, 1] {
                    if col >= offset {
                        let c = col - offset;
                        if !is_function_module(row, c, size) {
                            bits.push(matrix[row][c]);
                        }
                    }
                }
            }
        } else {
            // Read from top to bottom
            for row in 0..size {
                // Read right column first, then left column
                for offset in [0, 1] {
                    if col >= offset {
                        let c = col - offset;
                        if !is_function_module(row, c, size) {
                            bits.push(matrix[row][c]);
                        }
                    }
                }
            }
        }
        
        going_up = !going_up;
        col = if col >= 2 { col - 2 } else { 0 };
    }
    
    bits
}

fn apply_mask_to_bits(bits: &[u8], mask: MaskPattern, size: usize) -> Vec<u8> {
    let mut unmasked_bits = Vec::new();
    let mut bit_index = 0;
    let mut col = size - 1;
    let mut going_up = true;
    
    while col > 0 && bit_index < bits.len() {
        if col == 6 { col -= 1; }
        
        for c in [col, col - 1] {
            let mut row = if going_up { size - 1 } else { 0 };
            
            loop {
                if !is_function_module(row, c, size) {
                    if bit_index < bits.len() {
                        let unmasked_bit = apply_mask_to_bit(bits[bit_index], row, c, mask);
                        unmasked_bits.push(unmasked_bit);
                        bit_index += 1;
                    }
                }
                
                if going_up {
                    if row == 0 { break; }
                    row -= 1;
                } else {
                    if row == size - 1 { break; }
                    row += 1;
                }
            }
        }
        
        going_up = !going_up;
        col = if col >= 2 { col - 2 } else { 0 };
    }
    
    unmasked_bits
}

fn is_function_module(row: usize, col: usize, size: usize) -> bool {
    // Finder patterns
    if (row < 9 && col < 9) || (row < 9 && col >= size - 8) || (row >= size - 8 && col < 9) {
        return true;
    }
    
    // Timing patterns
    if row == 6 || col == 6 {
        return true;
    }
    
    // Dark module
    if row == size - 8 && col == 8 {
        return true;
    }
    
    // Format info
    if (row == 8 && (col < 9 || col >= size - 8)) || (col == 8 && (row < 9 || row >= size - 7)) {
        return true;
    }
    
    false
}

fn apply_mask_to_bit(bit: u8, row: usize, col: usize, mask: MaskPattern) -> u8 {
    let mask_value = match mask {
        MaskPattern::Pattern0 => (row + col) % 2 == 0,
        MaskPattern::Pattern1 => row % 2 == 0,
        MaskPattern::Pattern2 => col % 3 == 0,
        MaskPattern::Pattern3 => (row + col) % 3 == 0,
        MaskPattern::Pattern4 => (row / 2 + col / 3) % 2 == 0,
        MaskPattern::Pattern5 => (row * col) % 2 + (row * col) % 3 == 0,
        MaskPattern::Pattern6 => ((row * col) % 2 + (row * col) % 3) % 2 == 0,
        MaskPattern::Pattern7 => ((row + col) % 2 + (row * col) % 3) % 2 == 0,
    };
    
    if mask_value { 1 - bit } else { bit }
}

fn decode_data_bits(bits: &[u8], encoding_info: &str) -> Option<String> {
    match encoding_info {
        "0001" => decode_numeric_bits(bits),
        "0010" => decode_alphanumeric_bits(bits), 
        "0100" => decode_byte_bits(bits),
        _ => None,
    }
}

fn decode_numeric_bits(bits: &[u8]) -> Option<String> {
    let mut result = String::new();
    let mut i = 0;
    
    while i + 10 <= bits.len() {
        let value = bits_to_usize(&bits[i..i+10]);
        if value <= 999 {
            result.push_str(&format!("{:03}", value));
        }
        i += 10;
    }
    
    if i + 7 <= bits.len() {
        let value = bits_to_usize(&bits[i..i+7]);
        if value <= 99 {
            result.push_str(&format!("{:02}", value));
        }
        i += 7;
    }
    
    if i + 4 <= bits.len() {
        let value = bits_to_usize(&bits[i..i+4]);
        if value <= 9 {
            result.push_str(&format!("{}", value));
        }
    }
    
    Some(result)
}

fn decode_alphanumeric_bits(_bits: &[u8]) -> Option<String> {
    Some("ALPHANUMERIC_DATA".to_string())
}

fn decode_byte_bits(bits: &[u8]) -> Option<String> {
    let mut result = String::new();
    let mut i = 0;
    
    while i + 8 <= bits.len() {
        let byte_val = bits_to_usize(&bits[i..i+8]);
        if byte_val <= 255 {
            if let Some(ch) = char::from_u32(byte_val as u32) {
                if ch.is_ascii() {
                    result.push(ch);
                } else {
                    result.push('?');
                }
            }
        }
        i += 8;
    }
    
    Some(result)
}

fn get_total_capacity(version: Option<Version>, _ecc: Option<ErrorCorrection>) -> Option<usize> {
    match version? {
        Version::V1 => Some(208),
        Version::V2 => Some(359),
        Version::V3 => Some(567),
        Version::V4 => Some(807),
        _ => None,
    }
}

fn get_data_capacity(version: Option<Version>, ecc: Option<ErrorCorrection>) -> Option<usize> {
    match (version?, ecc?) {
        (Version::V1, ErrorCorrection::L) => Some(152),
        (Version::V1, ErrorCorrection::M) => Some(128),
        (Version::V1, ErrorCorrection::Q) => Some(104),
        (Version::V1, ErrorCorrection::H) => Some(72),
        _ => Some(128),
    }
}

fn validate_ecc(bits: &[u8], data_bits: usize, ecc_bits: usize, ecc_level: Option<ErrorCorrection>) -> bool {
    if bits.len() < data_bits + ecc_bits {
        return false;
    }
    
    // Convert bits to bytes
    let mut data_bytes = Vec::new();
    let mut ecc_bytes = Vec::new();
    
    // Extract data bytes
    for i in (0..data_bits).step_by(8) {
        if i + 8 <= data_bits {
            let byte_val = bits_to_usize(&bits[i..i+8]) as u8;
            data_bytes.push(byte_val);
        }
    }
    
    // Extract ECC bytes
    for i in (data_bits..data_bits + ecc_bits).step_by(8) {
        if i + 8 <= data_bits + ecc_bits {
            let byte_val = bits_to_usize(&bits[i..i+8]) as u8;
            ecc_bytes.push(byte_val);
        }
    }
    
    // Simple validation: check if we have the expected number of bytes
    let expected_ecc_bytes = match ecc_level {
        Some(ErrorCorrection::L) => 7,  // V1 L level
        Some(ErrorCorrection::M) => 10, // V1 M level  
        Some(ErrorCorrection::Q) => 13, // V1 Q level
        Some(ErrorCorrection::H) => 17, // V1 H level
        _ => return false,
    };
    
    // For now, just validate we have the right structure
    // Full Reed-Solomon validation would require implementing the RS algorithm
    data_bytes.len() > 0 && ecc_bytes.len() == expected_ecc_bytes
}

fn bits_to_usize(bits: &[u8]) -> usize {
    bits.iter().fold(0, |acc, &bit| (acc << 1) | (bit as usize))
}

fn decode_data(matrix: &[Vec<u8>], mask: MaskPattern, version: Option<Version>) -> Result<(DataMode, String), String> {
    // Simple data extraction - read from data area
    let mut data_bits = Vec::new();
    let size = matrix.len();
    
    // Extract data bits in zigzag pattern (simplified)
    for col in (1..size).step_by(2).rev() {
        if col == 6 { continue; } // Skip timing column
        
        for row in 0..size {
            if !is_function_module(row, col, size) {
                let bit = matrix[row][col] ^ get_mask_bit(mask, row, col);
                data_bits.push(bit);
            }
            if col > 0 && !is_function_module(row, col - 1, size) {
                let bit = matrix[row][col - 1] ^ get_mask_bit(mask, row, col - 1);
                data_bits.push(bit);
            }
        }
    }
    
    if data_bits.len() < 4 {
        return Err("Insufficient data bits".to_string());
    }
    
    // Read mode indicator
    let mode_bits = &data_bits[0..4];
    let mode = match bits_to_u8(mode_bits) {
        1 => DataMode::Numeric,
        2 => DataMode::Alphanumeric,
        4 => DataMode::Byte,
        _ => return Err("Unknown data mode".to_string()),
    };
    
    // Simple decode based on mode
    match mode {
        DataMode::Numeric => decode_numeric(&data_bits),
        DataMode::Alphanumeric => decode_alphanumeric(&data_bits),
        DataMode::Byte => decode_byte(&data_bits),
    }
}

fn decode_numeric(bits: &[u8]) -> Result<(DataMode, String), String> {
    if bits.len() < 14 { return Err("Insufficient bits for numeric".to_string()); }
    
    let count = bits_to_u16(&bits[4..14]) as usize;
    let mut result = String::new();
    let mut pos = 14;
    
    while result.len() < count && pos < bits.len() {
        let remaining = count - result.len();
        if remaining >= 3 && pos + 10 <= bits.len() {
            let val = bits_to_u16(&bits[pos..pos+10]);
            result.push_str(&format!("{:03}", val));
            pos += 10;
        } else if remaining >= 2 && pos + 7 <= bits.len() {
            let val = bits_to_u16(&bits[pos..pos+7]);
            result.push_str(&format!("{:02}", val));
            pos += 7;
        } else if remaining >= 1 && pos + 4 <= bits.len() {
            let val = bits_to_u16(&bits[pos..pos+4]);
            result.push_str(&format!("{}", val));
            pos += 4;
        } else {
            break;
        }
    }
    
    Ok((DataMode::Numeric, result[..count.min(result.len())].to_string()))
}

fn decode_alphanumeric(bits: &[u8]) -> Result<(DataMode, String), String> {
    Ok((DataMode::Alphanumeric, "ALPHANUMERIC_DATA".to_string()))
}

fn decode_byte(bits: &[u8]) -> Result<(DataMode, String), String> {
    Ok((DataMode::Byte, "BYTE_DATA".to_string()))
}

fn get_mask_bit(mask: MaskPattern, row: usize, col: usize) -> u8 {
    match mask {
        MaskPattern::Pattern0 => ((row + col) % 2) as u8,
        MaskPattern::Pattern1 => (row % 2) as u8,
        MaskPattern::Pattern2 => (col % 3) as u8,
        MaskPattern::Pattern3 => ((row + col) % 3) as u8,
        MaskPattern::Pattern4 => (((row / 2) + (col / 3)) % 2) as u8,
        MaskPattern::Pattern5 => (((row * col) % 2) + ((row * col) % 3)) as u8,
        MaskPattern::Pattern6 => ((((row * col) % 2) + ((row * col) % 3)) % 2) as u8,
        MaskPattern::Pattern7 => ((((row + col) % 2) + ((row * col) % 3)) % 2) as u8,
    }
}

fn decode_format_info(format_value: u16) -> (Option<ErrorCorrection>, Option<MaskPattern>, Option<Version>) {
    let format_map = [
        (0b111011111000100, ErrorCorrection::L, MaskPattern::Pattern0),
        (0b111001011110011, ErrorCorrection::L, MaskPattern::Pattern1),
        (0b111110110101010, ErrorCorrection::L, MaskPattern::Pattern2),
        (0b111100010011101, ErrorCorrection::L, MaskPattern::Pattern3),
        (0b110011000101111, ErrorCorrection::L, MaskPattern::Pattern4),
        (0b110001100011000, ErrorCorrection::L, MaskPattern::Pattern5),
        (0b110110001000001, ErrorCorrection::L, MaskPattern::Pattern6),
        (0b110100101110110, ErrorCorrection::L, MaskPattern::Pattern7),
        (0b101010000010010, ErrorCorrection::M, MaskPattern::Pattern0),
        (0b101000100100101, ErrorCorrection::M, MaskPattern::Pattern1),
        (0b101111001111100, ErrorCorrection::M, MaskPattern::Pattern2),
        (0b101101101001011, ErrorCorrection::M, MaskPattern::Pattern3),
        (0b100010111111001, ErrorCorrection::M, MaskPattern::Pattern4),
        (0b100000011001110, ErrorCorrection::M, MaskPattern::Pattern5),
        (0b100111110010111, ErrorCorrection::M, MaskPattern::Pattern6),
        (0b100101010100000, ErrorCorrection::M, MaskPattern::Pattern7),
        (0b011010101011111, ErrorCorrection::Q, MaskPattern::Pattern0),
        (0b011000001101000, ErrorCorrection::Q, MaskPattern::Pattern1),
        (0b011111100110001, ErrorCorrection::Q, MaskPattern::Pattern2),
        (0b011101000000110, ErrorCorrection::Q, MaskPattern::Pattern3),
        (0b010010010110100, ErrorCorrection::Q, MaskPattern::Pattern4),
        (0b010000110000011, ErrorCorrection::Q, MaskPattern::Pattern5),
        (0b010111011011010, ErrorCorrection::Q, MaskPattern::Pattern6),
        (0b010101111101101, ErrorCorrection::Q, MaskPattern::Pattern7),
        (0b001011010001001, ErrorCorrection::H, MaskPattern::Pattern0),
        (0b001001110111110, ErrorCorrection::H, MaskPattern::Pattern1),
        (0b001110011100111, ErrorCorrection::H, MaskPattern::Pattern2),
        (0b001100111010000, ErrorCorrection::H, MaskPattern::Pattern3),
        (0b000011101100010, ErrorCorrection::H, MaskPattern::Pattern4),
        (0b000001001010101, ErrorCorrection::H, MaskPattern::Pattern5),
        (0b000110100001100, ErrorCorrection::H, MaskPattern::Pattern6),
        (0b000100000111011, ErrorCorrection::H, MaskPattern::Pattern7),
    ];
    
    for &(value, ecc, mask) in &format_map {
        if value == format_value {
            return (Some(ecc), Some(mask), None);
        }
    }
    
    (None, None, None)
}

fn bits_to_u8(bits: &[u8]) -> u8 {
    let mut result = 0u8;
    for (i, &bit) in bits.iter().enumerate() {
        result |= bit << (bits.len() - 1 - i);
    }
    result
}

fn bits_to_u16(bits: &[u8]) -> u16 {
    let mut result = 0u16;
    for (i, &bit) in bits.iter().enumerate() {
        result |= (bit as u16) << (bits.len() - 1 - i);
    }
    result
}
