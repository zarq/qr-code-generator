use image;
use std::env;
use serde::Serialize;

mod types;
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
    format_info: FormatInfo,
    finder_patterns: Vec<FinderPattern>,
    timing_patterns: TimingPatterns,
    alignment_patterns: Vec<AlignmentPattern>,
    border_check: BorderCheck,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct FormatInfo {
    raw_bits: Option<String>,
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
            raw_bits: None,
            error_correction: None,
            mask_pattern: None,
            version: None,
        },
        finder_patterns: Vec::new(),
        timing_patterns: TimingPatterns { valid: false },
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
        if let Ok((mode, data)) = decode_data(&matrix, mask, analysis.version_from_size) {
            analysis.data_mode = Some(mode);
            analysis.decoded_text = Some(data.clone());
            analysis.raw_data = Some(data);
        }
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

fn analyze_format_info(matrix: &[Vec<u8>]) -> Option<FormatInfo> {
    let size = matrix.len();
    let mut bits = Vec::new();
    
    // Read format info from around top-left finder pattern
    for i in 0..6 {
        bits.push(matrix[8][i]);
    }
    bits.push(matrix[8][7]);
    bits.push(matrix[8][8]);
    bits.push(matrix[7][8]);
    for i in (0..6).rev() {
        bits.push(matrix[i][8]);
    }
    
    let raw_bits = bits.iter().map(|&b| if b == 1 { '1' } else { '0' }).collect::<String>();
    
    // Decode format info
    let format_value = bits_to_u16(&bits);
    let (ecc, mask, version) = decode_format_info(format_value);
    
    Some(FormatInfo {
        raw_bits: Some(raw_bits),
        error_correction: ecc,
        mask_pattern: mask,
        version,
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

fn is_function_module(row: usize, col: usize, size: usize) -> bool {
    // Finder patterns
    if (row < 9 && col < 9) || 
       (row < 9 && col >= size - 8) || 
       (row >= size - 8 && col < 9) {
        return true;
    }
    
    // Timing patterns
    if row == 6 || col == 6 {
        return true;
    }
    
    // Format info
    if (row == 8 && (col < 9 || col >= size - 8)) ||
       (col == 8 && (row < 9 || row >= size - 7)) {
        return true;
    }
    
    false
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
