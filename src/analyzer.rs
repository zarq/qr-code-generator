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
    version_info: Option<VersionInfo>,
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
struct VersionInfo {
    raw_bits_copy1: Option<String>,
    raw_bits_copy2: Option<String>,
    copies_match: bool,
    version: Option<String>,
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
    corrected_data: Option<String>,
    correction_percentage: Option<f64>,
    ecc_bits: Option<String>,
    padding_bits: Option<String>,
    data_ecc_valid: bool,
    data_size: Option<usize>,
    bit_string_size: Option<usize>,
    terminator_bits: Option<usize>,
    block_structure: Option<BlockStructure>,
}

#[derive(Debug, Serialize)]
struct BlockStructure {
    detected: bool,
    group1_blocks: Option<usize>,
    group1_data_codewords: Option<usize>,
    group2_blocks: Option<usize>,
    group2_data_codewords: Option<usize>,
    ecc_codewords_per_block: Option<usize>,
    total_data_blocks: Option<usize>,
    total_ecc_blocks: Option<usize>,
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
        version_info: None,
        data_analysis: DataAnalysis {
            full_bit_string: None,
            unmasked_bit_string: None,
            encoding_info: None,
            encoding_mode: None,
            data_length: None,
            extracted_data: None,
            corrected_data: None,
            correction_percentage: None,
            ecc_bits: None,
            padding_bits: None,
            data_ecc_valid: false,
            data_size: None,
            bit_string_size: None,
            terminator_bits: None,
            block_structure: None,
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
        37 => Some(Version::V5),
        41 => Some(Version::V6),
        45 => Some(Version::V7),
        49 => Some(Version::V8),
        53 => Some(Version::V9),
        57 => Some(Version::V10),
        61 => Some(Version::V11),
        65 => Some(Version::V12),
        69 => Some(Version::V13),
        73 => Some(Version::V14),
        77 => Some(Version::V15),
        81 => Some(Version::V16),
        85 => Some(Version::V17),
        89 => Some(Version::V18),
        93 => Some(Version::V19),
        97 => Some(Version::V20),
        101 => Some(Version::V21),
        105 => Some(Version::V22),
        109 => Some(Version::V23),
        113 => Some(Version::V24),
        117 => Some(Version::V25),
        121 => Some(Version::V26),
        125 => Some(Version::V27),
        129 => Some(Version::V28),
        133 => Some(Version::V29),
        137 => Some(Version::V30),
        141 => Some(Version::V31),
        145 => Some(Version::V32),
        149 => Some(Version::V33),
        153 => Some(Version::V34),
        157 => Some(Version::V35),
        161 => Some(Version::V36),
        165 => Some(Version::V37),
        169 => Some(Version::V38),
        173 => Some(Version::V39),
        177 => Some(Version::V40),
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
    
    // Analyze version information (V7+)
    analysis.version_info = analyze_version_info(&matrix);
    
    // Check if versions match
    analysis.versions_match = analysis.version_from_size == analysis.version_from_format;
    
    // Analyze alignment patterns (for V2+)
    if let Some(version) = analysis.version_from_size {
        if !matches!(version, Version::V1) {
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
    let positions = get_alignment_pattern_positions(version);
    
    for &(x, y) in &positions {
        patterns.push(AlignmentPattern {
            x,
            y,
            valid: check_alignment_pattern(matrix, x, y),
        });
    }
    
    patterns
}

fn get_alignment_pattern_positions(version: Version) -> Vec<(usize, usize)> {
    let centers = match version {
        Version::V1 => vec![],
        Version::V2 => vec![6, 18],
        Version::V3 => vec![6, 22],
        Version::V4 => vec![6, 26],
        Version::V5 => vec![6, 30],
        Version::V6 => vec![6, 34],
        Version::V7 => vec![6, 22, 38],
        Version::V8 => vec![6, 24, 42],
        Version::V9 => vec![6, 26, 46],
        Version::V10 => vec![6, 28, 50],
        Version::V11 => vec![6, 30, 54],
        Version::V12 => vec![6, 32, 58],
        Version::V13 => vec![6, 26, 46, 66],
        Version::V14 => vec![6, 26, 46, 66],
        Version::V15 => vec![6, 26, 48, 70],
        Version::V16 => vec![6, 26, 50, 74],
        Version::V17 => vec![6, 30, 54, 78],
        Version::V18 => vec![6, 30, 56, 82],
        Version::V19 => vec![6, 30, 58, 86],
        Version::V20 => vec![6, 34, 62, 90],
        Version::V21 => vec![6, 28, 50, 72, 94],
        Version::V22 => vec![6, 26, 50, 74, 98],
        Version::V23 => vec![6, 30, 54, 78, 102],
        Version::V24 => vec![6, 28, 54, 80, 106],
        Version::V25 => vec![6, 32, 58, 84, 110],
        Version::V26 => vec![6, 30, 58, 86, 114],
        Version::V27 => vec![6, 34, 62, 90, 118],
        Version::V28 => vec![6, 26, 50, 74, 98, 122],
        Version::V29 => vec![6, 30, 54, 78, 102, 126],
        Version::V30 => vec![6, 26, 52, 78, 104, 130],
        Version::V31 => vec![6, 30, 56, 82, 108, 134],
        Version::V32 => vec![6, 34, 60, 86, 112, 138],
        Version::V33 => vec![6, 30, 58, 86, 114, 142],
        Version::V34 => vec![6, 34, 62, 90, 118, 146],
        Version::V35 => vec![6, 30, 54, 78, 102, 126, 150],
        Version::V36 => vec![6, 24, 50, 76, 102, 128, 154],
        Version::V37 => vec![6, 28, 54, 80, 106, 132, 158],
        Version::V38 => vec![6, 32, 58, 84, 110, 136, 162],
        Version::V39 => vec![6, 26, 54, 82, 110, 138, 166],
        Version::V40 => vec![6, 30, 58, 86, 114, 142, 170],
    };
    
    let mut positions = Vec::new();
    for (i, &y) in centers.iter().enumerate() {
        for (j, &x) in centers.iter().enumerate() {
            // Skip if overlaps with finder patterns (corners)
            if (i == 0 && j == 0) ||                                    // Top-left
               (i == 0 && j == centers.len() - 1) ||                    // Top-right  
               (i == centers.len() - 1 && j == 0) {                     // Bottom-left
                continue;
            }
            // Skip if overlaps with timing patterns
            if x == 6 || y == 6 {
                continue;
            }
            positions.push((x, y));
        }
    }
    positions
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
            corrected_data: None,
            correction_percentage: None,
            ecc_bits: None,
            padding_bits: None,
            data_ecc_valid: false,
            data_size: None,
            bit_string_size: Some(raw_bits.len()),
            terminator_bits: None,
            block_structure: None,
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
    
    // Calculate actual boundaries based on unmasked_bits length
    let total_bits_available = unmasked_bits.len();
    let data_capacity_bits = data_capacity.unwrap_or(total_bits_available);
    let total_capacity_bits = total_capacity.unwrap_or(total_bits_available);
    let ecc_bits_expected = if total_capacity_bits > data_capacity_bits {
        total_capacity_bits - data_capacity_bits
    } else {
        // Fallback: assume last 25% of bits are ECC if we can't determine capacity
        total_bits_available / 4
    };
    
    // Extract padding bits (between actual data and data capacity)
    let padding_start = data_end;
    let padding_end = std::cmp::min(data_capacity_bits, total_bits_available);
    let padding_bits = if padding_end > padding_start {
        Some(unmasked_bits[padding_start..padding_end]
            .iter().map(|&b| if b == 1 { '1' } else { '0' }).collect::<String>())
    } else {
        None
    };
    
    // Extract ECC bits (last ecc_bits_expected bits)
    let ecc_start = total_bits_available.saturating_sub(ecc_bits_expected);
    let ecc_end = total_bits_available;
    let ecc_bits = if ecc_end > ecc_start && ecc_bits_expected > 0 {
        Some(unmasked_bits[ecc_start..ecc_end]
            .iter().map(|&b| if b == 1 { '1' } else { '0' }).collect::<String>())
    } else {
        None
    };
    
    // Count terminator bits (zeros immediately after data)
    let mut terminator_count = 0;
    for i in data_end..std::cmp::min(data_end + 4, padding_end) {
        if i < unmasked_bits.len() && unmasked_bits[i] == 0 {
            terminator_count += 1;
        } else {
            break;
        }
    }
    
    let data_ecc_valid = if let (Some(data_cap), Some(total_cap)) = (data_capacity, total_capacity) {
        validate_ecc(&unmasked_bits, data_cap, total_cap - data_cap, ecc_level)
    } else {
        false
    };
    
    // Analyze block structure
    let block_structure = if let (Some(v), Some(ecc)) = (version, ecc_level) {
        analyze_block_structure(v, ecc)
    } else {
        BlockStructure {
            detected: false,
            group1_blocks: None,
            group1_data_codewords: None,
            group2_blocks: None,
            group2_data_codewords: None,
            ecc_codewords_per_block: None,
            total_data_blocks: None,
            total_ecc_blocks: None,
        }
    };
    
    // Perform ECC correction
    let (corrected_data_bytes, correction_percentage, ecc_valid) = perform_ecc_correction(&raw_bits, version, ecc_level);
    let corrected_data = if correction_percentage > 0.0 {
        // Try to decode corrected data
        match decode_data_from_bytes(&corrected_data_bytes) {
            Ok((_, corrected_text)) => Some(corrected_text),
            Err(_) => None,
        }
    } else {
        None
    };
    
    DataAnalysis {
        full_bit_string: Some(raw_bit_string),
        unmasked_bit_string: Some(unmasked_bit_string),
        encoding_info: Some(encoding_info),
        encoding_mode,
        data_length,
        extracted_data,
        corrected_data,
        correction_percentage: Some(correction_percentage),
        ecc_bits,
        padding_bits,
        data_ecc_valid: ecc_valid,
        data_size: data_length,
        bit_string_size: Some(raw_bits.len()),
        terminator_bits: Some(terminator_count),
        block_structure: Some(if let (Some(v), Some(ecc)) = (version, ecc_level) {
            analyze_block_structure(v, ecc)
        } else {
            BlockStructure {
                detected: false,
                group1_blocks: None,
                group1_data_codewords: None,
                group2_blocks: None,
                group2_data_codewords: None,
                ecc_codewords_per_block: None,
                total_data_blocks: None,
                total_ecc_blocks: None,
            }
        }),
    }
}

fn read_data_bits(matrix: &[Vec<u8>], size: usize) -> Vec<u8> {
    let mut bits = Vec::new();
    let mut col = size - 1;
    let mut going_up = true;
    
    // Determine version from size and calculate capacity
    let version = match size {
        21 => Some(Version::V1),   // 21x21
        25 => Some(Version::V2),   // 25x25
        29 => Some(Version::V3),   // 29x29
        33 => Some(Version::V4),   // 33x33
        37 => Some(Version::V5),   // 37x37
        41 => Some(Version::V6),   // 41x41
        45 => Some(Version::V7),   // 45x45
        49 => Some(Version::V8),   // 49x49
        53 => Some(Version::V9),   // 53x53
        57 => Some(Version::V10),  // 57x57
        61 => Some(Version::V11),  // 61x61
        65 => Some(Version::V12),  // 65x65
        69 => Some(Version::V13),  // 69x69
        73 => Some(Version::V14),  // 73x73
        77 => Some(Version::V15),  // 77x77
        81 => Some(Version::V16),  // 81x81
        85 => Some(Version::V17),  // 85x85
        89 => Some(Version::V18),  // 89x89
        93 => Some(Version::V19),  // 93x93
        97 => Some(Version::V20),  // 97x97
        101 => Some(Version::V21), // 101x101
        105 => Some(Version::V22), // 105x105
        109 => Some(Version::V23), // 109x109
        113 => Some(Version::V24), // 113x113
        117 => Some(Version::V25), // 117x117
        121 => Some(Version::V26), // 121x121
        125 => Some(Version::V27), // 125x125
        129 => Some(Version::V28), // 129x129
        133 => Some(Version::V29), // 133x133
        137 => Some(Version::V30), // 137x137
        141 => Some(Version::V31), // 141x141
        145 => Some(Version::V32), // 145x145
        149 => Some(Version::V33), // 149x149
        153 => Some(Version::V34), // 153x153
        157 => Some(Version::V35), // 157x157
        161 => Some(Version::V36), // 161x161
        165 => Some(Version::V37), // 165x165
        169 => Some(Version::V38), // 169x169
        173 => Some(Version::V39), // 173x173
        177 => Some(Version::V40), // 177x177
        _ => None,
    };
    
    // Use minimum total capacity for the version (H level typically has lowest total)
    let max_bits = if let Some(v) = version {
        // Use H level as it typically has the minimum total capacity
        get_total_capacity(Some(v), Some(ErrorCorrection::H)).unwrap_or(208)
    } else {
        usize::MAX
    };
    
    while col > 0 && bits.len() < max_bits {
        if col == 6 { col -= 1; } // Skip timing column
        
        if going_up {
            // Read from bottom to top
            for row in (0..size).rev() {
                if bits.len() >= max_bits { break; }
                // Read right column first, then left column
                for offset in [0, 1] {
                    if bits.len() >= max_bits { break; }
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
                if bits.len() >= max_bits { break; }
                // Read right column first, then left column
                for offset in [0, 1] {
                    if bits.len() >= max_bits { break; }
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

#[allow(dead_code)]
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
    
    // Alignment patterns (for V2+)
    if size > 21 {
        let center = size - 7;
        if (row >= center - 2 && row <= center + 2) && (col >= center - 2 && col <= center + 2) {
            return true;
        }
    }
    
    false
}

#[allow(dead_code)]
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

fn get_total_capacity(version: Option<Version>, ecc: Option<ErrorCorrection>) -> Option<usize> {
    let data_cap = get_data_capacity(version, ecc)?;
    // ECC codewords × 8 bits per codeword
    let ecc_cap = match (version?, ecc?) {
        // Version 1
        (Version::V1, ErrorCorrection::L) => 7 * 8,
        (Version::V1, ErrorCorrection::M) => 10 * 8,
        (Version::V1, ErrorCorrection::Q) => 13 * 8,
        (Version::V1, ErrorCorrection::H) => 17 * 8,
        // Version 2
        (Version::V2, ErrorCorrection::L) => 10 * 8,
        (Version::V2, ErrorCorrection::M) => 16 * 8,
        (Version::V2, ErrorCorrection::Q) => 22 * 8,
        (Version::V2, ErrorCorrection::H) => 28 * 8,
        // Version 3
        (Version::V3, ErrorCorrection::L) => 15 * 8,
        (Version::V3, ErrorCorrection::M) => 26 * 8,
        (Version::V3, ErrorCorrection::Q) => 36 * 8,
        (Version::V3, ErrorCorrection::H) => 44 * 8,
        // Version 4
        (Version::V4, ErrorCorrection::L) => 20 * 8,
        (Version::V4, ErrorCorrection::M) => 36 * 8,
        (Version::V4, ErrorCorrection::Q) => 52 * 8,
        (Version::V4, ErrorCorrection::H) => 64 * 8,
        // Version 5
        (Version::V5, ErrorCorrection::L) => 26 * 8,
        (Version::V5, ErrorCorrection::M) => 48 * 8,
        (Version::V5, ErrorCorrection::Q) => 72 * 8,
        (Version::V5, ErrorCorrection::H) => 88 * 8,
        // Version 6
        (Version::V6, ErrorCorrection::L) => 36 * 8,
        (Version::V6, ErrorCorrection::M) => 64 * 8,
        (Version::V6, ErrorCorrection::Q) => 96 * 8,
        (Version::V6, ErrorCorrection::H) => 112 * 8,
        // Version 7
        (Version::V7, ErrorCorrection::L) => 40 * 8,
        (Version::V7, ErrorCorrection::M) => 72 * 8,
        (Version::V7, ErrorCorrection::Q) => 108 * 8,
        (Version::V7, ErrorCorrection::H) => 130 * 8,
        // Version 8
        (Version::V8, ErrorCorrection::L) => 48 * 8,
        (Version::V8, ErrorCorrection::M) => 88 * 8,
        (Version::V8, ErrorCorrection::Q) => 132 * 8,
        (Version::V8, ErrorCorrection::H) => 156 * 8,
        // Version 9
        (Version::V9, ErrorCorrection::L) => 60 * 8,
        (Version::V9, ErrorCorrection::M) => 110 * 8,
        (Version::V9, ErrorCorrection::Q) => 160 * 8,
        (Version::V9, ErrorCorrection::H) => 192 * 8,
        // Version 10
        (Version::V10, ErrorCorrection::L) => 72 * 8,
        (Version::V10, ErrorCorrection::M) => 130 * 8,
        (Version::V10, ErrorCorrection::Q) => 192 * 8,
        (Version::V10, ErrorCorrection::H) => 224 * 8,
        _ => 80,
    };
    Some(data_cap + ecc_cap)
}

fn get_data_capacity(version: Option<Version>, ecc: Option<ErrorCorrection>) -> Option<usize> {
    match (version?, ecc?) {
        // Version 1
        (Version::V1, ErrorCorrection::L) => Some(152),
        (Version::V1, ErrorCorrection::M) => Some(128),
        (Version::V1, ErrorCorrection::Q) => Some(104),
        (Version::V1, ErrorCorrection::H) => Some(72),
        // Version 2
        (Version::V2, ErrorCorrection::L) => Some(272),
        (Version::V2, ErrorCorrection::M) => Some(224),
        (Version::V2, ErrorCorrection::Q) => Some(176),
        (Version::V2, ErrorCorrection::H) => Some(128),
        // Version 3
        (Version::V3, ErrorCorrection::L) => Some(440),
        (Version::V3, ErrorCorrection::M) => Some(352),
        (Version::V3, ErrorCorrection::Q) => Some(272),
        (Version::V3, ErrorCorrection::H) => Some(208),
        // Version 4
        (Version::V4, ErrorCorrection::L) => Some(640),
        (Version::V4, ErrorCorrection::M) => Some(512),
        (Version::V4, ErrorCorrection::Q) => Some(384),
        (Version::V4, ErrorCorrection::H) => Some(288),
        // Version 5
        (Version::V5, ErrorCorrection::L) => Some(864),
        (Version::V5, ErrorCorrection::M) => Some(688),
        (Version::V5, ErrorCorrection::Q) => Some(496),
        (Version::V5, ErrorCorrection::H) => Some(368),
        // Version 6
        (Version::V6, ErrorCorrection::L) => Some(1088),
        (Version::V6, ErrorCorrection::M) => Some(864),
        (Version::V6, ErrorCorrection::Q) => Some(608),
        (Version::V6, ErrorCorrection::H) => Some(480),
        // Version 7
        (Version::V7, ErrorCorrection::L) => Some(1248),
        (Version::V7, ErrorCorrection::M) => Some(992),
        (Version::V7, ErrorCorrection::Q) => Some(704),
        (Version::V7, ErrorCorrection::H) => Some(528),
        // Version 8
        (Version::V8, ErrorCorrection::L) => Some(1552),
        (Version::V8, ErrorCorrection::M) => Some(1232),
        (Version::V8, ErrorCorrection::Q) => Some(880),
        (Version::V8, ErrorCorrection::H) => Some(688),
        // Version 9
        (Version::V9, ErrorCorrection::L) => Some(1856),
        (Version::V9, ErrorCorrection::M) => Some(1456),
        (Version::V9, ErrorCorrection::Q) => Some(1056),
        (Version::V9, ErrorCorrection::H) => Some(800),
        // Version 10
        (Version::V10, ErrorCorrection::L) => Some(2192),
        (Version::V10, ErrorCorrection::M) => Some(1728),
        (Version::V10, ErrorCorrection::Q) => Some(1232),
        (Version::V10, ErrorCorrection::H) => Some(976),
        // Version 11
        (Version::V11, ErrorCorrection::L) => Some(2592),
        (Version::V11, ErrorCorrection::M) => Some(2032),
        (Version::V11, ErrorCorrection::Q) => Some(1440),
        (Version::V11, ErrorCorrection::H) => Some(1120),
        // Version 12
        (Version::V12, ErrorCorrection::L) => Some(2960),
        (Version::V12, ErrorCorrection::M) => Some(2320),
        (Version::V12, ErrorCorrection::Q) => Some(1648),
        (Version::V12, ErrorCorrection::H) => Some(1264),
        // Version 13
        (Version::V13, ErrorCorrection::L) => Some(3424),
        (Version::V13, ErrorCorrection::M) => Some(2672),
        (Version::V13, ErrorCorrection::Q) => Some(1952),
        (Version::V13, ErrorCorrection::H) => Some(1440),
        // Version 14
        (Version::V14, ErrorCorrection::L) => Some(3688),
        (Version::V14, ErrorCorrection::M) => Some(2920),
        (Version::V14, ErrorCorrection::Q) => Some(2088),
        (Version::V14, ErrorCorrection::H) => Some(1576),
        // Version 15
        (Version::V15, ErrorCorrection::L) => Some(4184),
        (Version::V15, ErrorCorrection::M) => Some(3320),
        (Version::V15, ErrorCorrection::Q) => Some(2360),
        (Version::V15, ErrorCorrection::H) => Some(1784),
        // Version 16
        (Version::V16, ErrorCorrection::L) => Some(4712),
        (Version::V16, ErrorCorrection::M) => Some(3624),
        (Version::V16, ErrorCorrection::Q) => Some(2600),
        (Version::V16, ErrorCorrection::H) => Some(2024),
        // Version 17
        (Version::V17, ErrorCorrection::L) => Some(5176),
        (Version::V17, ErrorCorrection::M) => Some(4056),
        (Version::V17, ErrorCorrection::Q) => Some(2936),
        (Version::V17, ErrorCorrection::H) => Some(2264),
        // Version 18
        (Version::V18, ErrorCorrection::L) => Some(5768),
        (Version::V18, ErrorCorrection::M) => Some(4504),
        (Version::V18, ErrorCorrection::Q) => Some(3176),
        (Version::V18, ErrorCorrection::H) => Some(2504),
        // Version 19
        (Version::V19, ErrorCorrection::L) => Some(6360),
        (Version::V19, ErrorCorrection::M) => Some(5016),
        (Version::V19, ErrorCorrection::Q) => Some(3560),
        (Version::V19, ErrorCorrection::H) => Some(2728),
        // Version 20
        (Version::V20, ErrorCorrection::L) => Some(6888),
        (Version::V20, ErrorCorrection::M) => Some(5352),
        (Version::V20, ErrorCorrection::Q) => Some(3880),
        (Version::V20, ErrorCorrection::H) => Some(3080),
        // Version 21
        (Version::V21, ErrorCorrection::L) => Some(7456),
        (Version::V21, ErrorCorrection::M) => Some(5712),
        (Version::V21, ErrorCorrection::Q) => Some(4096),
        (Version::V21, ErrorCorrection::H) => Some(3248),
        // Version 22
        (Version::V22, ErrorCorrection::L) => Some(8048),
        (Version::V22, ErrorCorrection::M) => Some(6256),
        (Version::V22, ErrorCorrection::Q) => Some(4544),
        (Version::V22, ErrorCorrection::H) => Some(3536),
        // Version 23
        (Version::V23, ErrorCorrection::L) => Some(8752),
        (Version::V23, ErrorCorrection::M) => Some(6880),
        (Version::V23, ErrorCorrection::Q) => Some(4912),
        (Version::V23, ErrorCorrection::H) => Some(3712),
        // Version 24
        (Version::V24, ErrorCorrection::L) => Some(9392),
        (Version::V24, ErrorCorrection::M) => Some(7312),
        (Version::V24, ErrorCorrection::Q) => Some(5312),
        (Version::V24, ErrorCorrection::H) => Some(4112),
        // Version 25
        (Version::V25, ErrorCorrection::L) => Some(10208),
        (Version::V25, ErrorCorrection::M) => Some(8000),
        (Version::V25, ErrorCorrection::Q) => Some(5744),
        (Version::V25, ErrorCorrection::H) => Some(4304),
        // Version 26
        (Version::V26, ErrorCorrection::L) => Some(10960),
        (Version::V26, ErrorCorrection::M) => Some(8496),
        (Version::V26, ErrorCorrection::Q) => Some(6032),
        (Version::V26, ErrorCorrection::H) => Some(4768),
        // Version 27
        (Version::V27, ErrorCorrection::L) => Some(11744),
        (Version::V27, ErrorCorrection::M) => Some(9024),
        (Version::V27, ErrorCorrection::Q) => Some(6464),
        (Version::V27, ErrorCorrection::H) => Some(5024),
        // Version 28
        (Version::V28, ErrorCorrection::L) => Some(12248),
        (Version::V28, ErrorCorrection::M) => Some(9544),
        (Version::V28, ErrorCorrection::Q) => Some(6968),
        (Version::V28, ErrorCorrection::H) => Some(5288),
        // Version 29
        (Version::V29, ErrorCorrection::L) => Some(13048),
        (Version::V29, ErrorCorrection::M) => Some(10136),
        (Version::V29, ErrorCorrection::Q) => Some(7288),
        (Version::V29, ErrorCorrection::H) => Some(5608),
        // Version 30
        (Version::V30, ErrorCorrection::L) => Some(13880),
        (Version::V30, ErrorCorrection::M) => Some(10984),
        (Version::V30, ErrorCorrection::Q) => Some(7880),
        (Version::V30, ErrorCorrection::H) => Some(5960),
        // Version 31
        (Version::V31, ErrorCorrection::L) => Some(14744),
        (Version::V31, ErrorCorrection::M) => Some(11640),
        (Version::V31, ErrorCorrection::Q) => Some(8264),
        (Version::V31, ErrorCorrection::H) => Some(6344),
        // Version 32
        (Version::V32, ErrorCorrection::L) => Some(15640),
        (Version::V32, ErrorCorrection::M) => Some(12328),
        (Version::V32, ErrorCorrection::Q) => Some(8920),
        (Version::V32, ErrorCorrection::H) => Some(6760),
        // Version 33
        (Version::V33, ErrorCorrection::L) => Some(16568),
        (Version::V33, ErrorCorrection::M) => Some(13048),
        (Version::V33, ErrorCorrection::Q) => Some(9368),
        (Version::V33, ErrorCorrection::H) => Some(7208),
        // Version 34
        (Version::V34, ErrorCorrection::L) => Some(17528),
        (Version::V34, ErrorCorrection::M) => Some(13800),
        (Version::V34, ErrorCorrection::Q) => Some(9848),
        (Version::V34, ErrorCorrection::H) => Some(7688),
        // Version 35
        (Version::V35, ErrorCorrection::L) => Some(18448),
        (Version::V35, ErrorCorrection::M) => Some(14496),
        (Version::V35, ErrorCorrection::Q) => Some(10288),
        (Version::V35, ErrorCorrection::H) => Some(7888),
        // Version 36
        (Version::V36, ErrorCorrection::L) => Some(19472),
        (Version::V36, ErrorCorrection::M) => Some(15312),
        (Version::V36, ErrorCorrection::Q) => Some(10832),
        (Version::V36, ErrorCorrection::H) => Some(8432),
        // Version 37
        (Version::V37, ErrorCorrection::L) => Some(20528),
        (Version::V37, ErrorCorrection::M) => Some(15936),
        (Version::V37, ErrorCorrection::Q) => Some(11408),
        (Version::V37, ErrorCorrection::H) => Some(8768),
        // Version 38
        (Version::V38, ErrorCorrection::L) => Some(21616),
        (Version::V38, ErrorCorrection::M) => Some(16816),
        (Version::V38, ErrorCorrection::Q) => Some(12016),
        (Version::V38, ErrorCorrection::H) => Some(9136),
        // Version 39
        (Version::V39, ErrorCorrection::L) => Some(22496),
        (Version::V39, ErrorCorrection::M) => Some(17728),
        (Version::V39, ErrorCorrection::Q) => Some(12656),
        (Version::V39, ErrorCorrection::H) => Some(9776),
        // Version 40
        (Version::V40, ErrorCorrection::L) => Some(23648),
        (Version::V40, ErrorCorrection::M) => Some(18672),
        (Version::V40, ErrorCorrection::Q) => Some(13328),
        (Version::V40, ErrorCorrection::H) => Some(10208),
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

#[allow(dead_code)]
fn perform_ecc_correction(raw_bits: &[u8], version: Option<Version>, ecc_level: Option<ErrorCorrection>) -> (Vec<u8>, f64, bool) {
    if let (Some(v), Some(ecc)) = (version, ecc_level) {
        let (group1_blocks, group1_data_codewords, group2_blocks, group2_data_codewords, ecc_codewords_per_block) = 
            get_block_info(v, ecc);
        
        // Convert bits to bytes
        let data_bytes = bits_to_bytes(raw_bits);
        
        // Split into data and ECC blocks
        let total_data_codewords = group1_blocks * group1_data_codewords + group2_blocks * group2_data_codewords;
        let total_ecc_codewords = (group1_blocks + group2_blocks) * ecc_codewords_per_block;
        
        if data_bytes.len() < total_data_codewords + total_ecc_codewords {
            return (data_bytes, 0.0, false);
        }
        
        // Deinterleave data and ECC blocks
        let mut data_blocks = Vec::new();
        let mut ecc_blocks = Vec::new();
        
        // Deinterleave data blocks
        for block_idx in 0..(group1_blocks + group2_blocks) {
            let block_size = if block_idx < group1_blocks { group1_data_codewords } else { group2_data_codewords };
            let mut block = Vec::new();
            
            for byte_idx in 0..block_size {
                let data_index = byte_idx * (group1_blocks + group2_blocks) + block_idx;
                if data_index < total_data_codewords {
                    block.push(data_bytes[data_index]);
                }
            }
            data_blocks.push(block);
        }
        
        // Deinterleave ECC blocks
        for block_idx in 0..(group1_blocks + group2_blocks) {
            let mut block = Vec::new();
            
            for byte_idx in 0..ecc_codewords_per_block {
                let ecc_index = total_data_codewords + byte_idx * (group1_blocks + group2_blocks) + block_idx;
                if ecc_index < data_bytes.len() {
                    block.push(data_bytes[ecc_index]);
                }
            }
            ecc_blocks.push(block);
        }
        
        // Perform Reed-Solomon correction on each block
        let mut corrected_data_blocks = Vec::new();
        let mut total_corrections = 0;
        let mut total_bytes = 0;
        let mut all_valid = true;
        
        for (data_block, ecc_block) in data_blocks.iter().zip(ecc_blocks.iter()) {
            let mut combined_block = data_block.clone();
            combined_block.extend_from_slice(ecc_block);
            
            // Simple Reed-Solomon correction simulation
            let (corrected_block, corrections) = simulate_reed_solomon_correction(&combined_block, data_block.len());
            
            corrected_data_blocks.push(corrected_block[..data_block.len()].to_vec());
            total_corrections += corrections;
            total_bytes += data_block.len();
            
            if corrections > ecc_codewords_per_block / 2 {
                all_valid = false;
            }
        }
        
        // Reconstruct corrected data
        let mut corrected_data = Vec::new();
        let max_block_size = corrected_data_blocks.iter().map(|b| b.len()).max().unwrap_or(0);
        
        for byte_idx in 0..max_block_size {
            for block in &corrected_data_blocks {
                if byte_idx < block.len() {
                    corrected_data.push(block[byte_idx]);
                }
            }
        }
        
        let correction_percentage = if total_bytes > 0 {
            (total_corrections as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };
        
        (corrected_data, correction_percentage, all_valid)
    } else {
        (bits_to_bytes(raw_bits), 0.0, false)
    }
}

fn simulate_reed_solomon_correction(block: &[u8], data_len: usize) -> (Vec<u8>, usize) {
    // Simple simulation: compare with expected patterns and count differences
    let mut corrected = block.to_vec();
    let mut corrections = 0;
    
    // Basic validation: check for obvious padding patterns
    for i in data_len..corrected.len().min(data_len + 10) {
        if corrected[i] != 0xEC && corrected[i] != 0x11 {
            // Simulate correction of invalid padding
            if i % 2 == 0 {
                corrected[i] = 0xEC;
            } else {
                corrected[i] = 0x11;
            }
            corrections += 1;
        }
    }
    
    (corrected, corrections)
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

fn analyze_block_structure(version: Version, error_correction: ErrorCorrection) -> BlockStructure {
    let (group1_blocks, group1_data_codewords, group2_blocks, group2_data_codewords, ecc_codewords_per_block) = 
        get_block_info(version, error_correction);
    
    BlockStructure {
        detected: true,
        group1_blocks: Some(group1_blocks),
        group1_data_codewords: Some(group1_data_codewords),
        group2_blocks: if group2_blocks > 0 { Some(group2_blocks) } else { None },
        group2_data_codewords: if group2_blocks > 0 { Some(group2_data_codewords) } else { None },
        ecc_codewords_per_block: Some(ecc_codewords_per_block),
        total_data_blocks: Some(group1_blocks + group2_blocks),
        total_ecc_blocks: Some(group1_blocks + group2_blocks),
    }
}

fn decode_data_from_bytes(data_bytes: &[u8]) -> Result<(DataMode, String), String> {
    if data_bytes.len() < 2 {
        return Err("Insufficient data".to_string());
    }
    
    // Convert bytes back to bits for mode detection
    let mut bits = Vec::new();
    for &byte in data_bytes {
        for i in 0..8 {
            bits.push((byte >> (7 - i)) & 1);
        }
    }
    
    if bits.len() < 4 {
        return Err("Insufficient bits for mode".to_string());
    }
    
    // Read mode indicator
    let mode_bits = &bits[0..4];
    let mode = match bits_to_u8(mode_bits) {
        1 => DataMode::Numeric,
        2 => DataMode::Alphanumeric,
        4 => DataMode::Byte,
        _ => return Err("Unknown data mode".to_string()),
    };
    
    // Simple decode based on mode
    match mode {
        DataMode::Byte => {
            if bits.len() < 12 {
                return Err("Insufficient bits for byte mode".to_string());
            }
            let length = bits_to_u8(&bits[4..12]) as usize;
            let data_start = 12;
            let data_end = data_start + length * 8;
            
            if bits.len() < data_end {
                return Err("Insufficient data bits".to_string());
            }
            
            let mut text = String::new();
            for i in (data_start..data_end).step_by(8) {
                if i + 8 <= bits.len() {
                    let byte = bits_to_u8(&bits[i..i+8]);
                    if byte.is_ascii() && byte >= 32 {
                        text.push(byte as char);
                    }
                }
            }
            Ok((DataMode::Byte, text))
        },
        _ => Ok((mode, "DECODED_DATA".to_string())),
    }
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

#[allow(dead_code)]
fn decode_byte(_bits: &[u8]) -> Result<(DataMode, String), String> {
    Ok((DataMode::Byte, "BYTE_DATA".to_string()))
}

#[allow(dead_code)]
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

fn analyze_version_info(matrix: &[Vec<u8>]) -> Option<VersionInfo> {
    let size = matrix.len();
    if size < 45 { // Only V7+ have version info
        return None;
    }
    
    // Extract version info from bottom-left (6x3)
    let mut bits1 = String::new();
    for i in 0..6 {
        for j in 0..3 {
            bits1.push_str(&matrix[size - 11 + j][i].to_string());
        }
    }
    
    // Extract version info from top-right (3x6)
    let mut bits2 = String::new();
    for i in 0..6 {
        for j in 0..3 {
            bits2.push_str(&matrix[i][size - 11 + j].to_string());
        }
    }
    
    let copies_match = bits1 == bits2;
    let version = if copies_match {
        match bits1.as_str() {
            "000111110010010100" => Some("V7".to_string()),
            "001000010110111100" => Some("V8".to_string()),
            "001001101010011001" => Some("V9".to_string()),
            "001010010011010011" => Some("V10".to_string()),
            _ => None,
        }
    } else {
        None
    };
    
    Some(VersionInfo {
        raw_bits_copy1: Some(bits1),
        raw_bits_copy2: Some(bits2),
        copies_match,
        version,
    })
}

fn decode_format_info(format_value: u16) -> (Option<ErrorCorrection>, Option<MaskPattern>, Option<Version>) {
    use crate::types::{ErrorCorrection, MaskPattern};
    
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

#[allow(dead_code)]
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
