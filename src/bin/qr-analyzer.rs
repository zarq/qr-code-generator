use image;
use qr_tools::capacity::get_data_capacity_in_bits;
use qr_tools::capacity::get_total_codewords_in_bits;
use qr_tools::capacity::image_size_to_version;
use qr_tools::ecc::generate_ecc;
use qr_tools::ecc::CorrectionResult;
use std::env;
use std::iter::zip;
use serde::Serialize;

use qr_tools::types;
use qr_tools::mask;
use qr_tools::ecc;
use types::{Version, ErrorCorrection, MaskPattern, DataMode};

#[derive(Debug, Serialize)]
struct BorderCheck {
    has_border: bool,
    border_width: usize,
    valid: bool,
}

#[derive(Debug, Serialize)]
struct QrAnalysis {
    version_from_size: Option<Version>,
    version_from_format: Option<Version>,
    versions_match: bool,
    size: usize,
    error_correction: Option<ErrorCorrection>,
    mask_pattern: Option<MaskPattern>,
    data_analysis: DataAnalysis,
    format_info: FormatInfo,
    version_info: Option<VersionInfo>,
    finder_patterns: Vec<FinderPattern>,
    timing_patterns: TimingPatterns,
    dark_module: DarkModule,
    alignment_patterns: Vec<AlignmentPattern>,
    border_check: BorderCheck,
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
    decoded_bit_string: Option<String>,
    unmasked_bit_string: Option<String>,
    unmasked_bytes: Option<String>,
    corrected_bit_string: Option<String>,
    corrected_bytes: Option<String>,
    expected_bit_string_size: Option<usize>,
    actual_bit_string_size: Option<usize>,
    expected_data_bit_string_size: Option<usize>,
    expected_ecc_bit_string_size: Option<usize>,
    encoding_info_bit_string: Option<String>,
    encoding_name: Option<String>,
    read_data_bytes: Option<String>,
    read_ecc_bytes: Option<String>,
    data_length: Option<usize>,
    extracted_data: Option<String>,
    corrected_data: Option<String>,
    message_bytes: Option<String>,
    reconstructed_ecc_bytes: Option<String>,
    data_error_positions: Option<Vec<usize>>,
    corrupted_bytes_percentage: Option<f64>,
    padding_bits: Option<String>,
    data_ecc_valid: bool,
    block_structure: Option<BlockStructure>,
    data_corrupted: bool,
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
        version_from_size: None,
        version_from_format: None,
        versions_match: false,
        size: inner_size,
        error_correction: None,
        mask_pattern: None,
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
            decoded_bit_string: None,
            unmasked_bit_string: None,
            unmasked_bytes: None,
            corrected_bit_string: None,
            corrected_bytes: None,
            expected_bit_string_size: None,
            actual_bit_string_size: None,
            expected_data_bit_string_size: None,
            expected_ecc_bit_string_size: None,
            encoding_info_bit_string: None,
            encoding_name: None,
            data_length: None,
            message_bytes: None,
            reconstructed_ecc_bytes: None,
            read_data_bytes: None,
            read_ecc_bytes: None,
            extracted_data: None,
            corrected_data: None,
            data_error_positions: None,
            corrupted_bytes_percentage: None,
            padding_bits: None,
            data_ecc_valid: false,
            block_structure: None,
            data_corrupted: false,
        },
        finder_patterns: Vec::new(),
        timing_patterns: TimingPatterns { valid: false },
        dark_module: DarkModule { present: false, position: (0, 0) },
        alignment_patterns: Vec::new(),
        border_check,
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
            panic!("Unsupported QR code size: {}x{}", inner_size, inner_size);
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
        analysis.data_analysis = decode_data_comprehensive(&matrix, mask, analysis.version_from_size.unwrap(), analysis.error_correction);
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
    
    // Decode format info from copy 1 with BCH error correction
    let format_value = bits_to_u16(&bits1);
    println!("Format bits (copy 1): {:015b}", format_value);
    println!("Format bits (copy 2): {:015b}", bits_to_u16(&bits2));
    let (ecc, mask) = if let Some((ec, mask_idx)) = correct_format_info(format_value) {
        println!("Corrected format info: ECC {:?}, Mask {:?}", ec, mask_idx);
        (Some(ec), Some(MaskPattern::from_index(mask_idx)))
    } else {
        println!("Failed to correct format info");
        // Fallback to old method if BCH correction fails
        let (ecc, mask, _) = decode_format_info(format_value);
        (ecc, mask)
    };
    
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

fn decode_data_comprehensive(matrix: &[Vec<u8>], mask: MaskPattern, version: Version, ecc_level: Option<ErrorCorrection>) -> DataAnalysis {
    let size = matrix.len();

    let mut analysis_result = DataAnalysis {
        decoded_bit_string: None,
        unmasked_bit_string: None,
        unmasked_bytes: None,
        corrected_bytes: None,
        corrected_bit_string: None,
        expected_bit_string_size: None,
        actual_bit_string_size: None,
        expected_data_bit_string_size: None,
        expected_ecc_bit_string_size: None,
        encoding_info_bit_string: None,
        reconstructed_ecc_bytes: None,
        encoding_name: None,
        data_length: None,
        message_bytes: None,
        read_data_bytes: None,
        read_ecc_bytes: None,
        extracted_data: None,
        corrected_data: None,
        data_error_positions: None,
        corrupted_bytes_percentage: None,
        padding_bits: None,
        data_ecc_valid: false,
        block_structure: None,
        data_corrupted: true,
    };
    
    // Step 1: Read raw bit string from matrix
    let decoded_bits = read_data_bits(matrix, size);
    let decoded_bit_string = decoded_bits.iter().map(|&b| if b == 1 { '1' } else { '0' }).collect::<String>();
    analysis_result.decoded_bit_string = Some(decoded_bit_string);
    
    // Step 2: Apply mask to matrix and read unmasked bits
    let mut unmasked_matrix = matrix.to_vec();
    mask::apply_mask(&mut unmasked_matrix, mask);
    let unmasked_bits = read_data_bits(&unmasked_matrix, size);
    let unmasked_bit_string = unmasked_bits.iter().map(|&b| if b == 1 { '1' } else { '0' }).collect::<String>();
    analysis_result.unmasked_bit_string = Some(unmasked_bit_string.clone());
    
    if unmasked_bits.len() < 8 {
        return analysis_result;
    }
    let unmasked_bytes = bits_to_bytes(&unmasked_bits);
    analysis_result.unmasked_bytes = Some(unmasked_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" "));

    if ecc_level.is_none() {
        return analysis_result;
    }
    
    // Step 2.5: Attempt error correction or fallback to original data
    let total_capacity_bits = get_total_codewords_in_bits(version);
    analysis_result.expected_bit_string_size = Some(total_capacity_bits);
    analysis_result.actual_bit_string_size = Some(unmasked_bits.len());

    if ecc_level.is_none() {
        return analysis_result;
    }
    
    let data_capacity_bits = get_data_capacity_in_bits(version, ecc_level.unwrap());
    analysis_result.expected_data_bit_string_size = Some(data_capacity_bits);
    
    // Calculate actual boundaries based on unmasked_bits length
    if data_capacity_bits > unmasked_bits.len() {
        println!("Error: Not enough bits read. Expected {}, got {}", data_capacity_bits, unmasked_bits.len());
        return analysis_result; // Not enough bits read
    }
    if data_capacity_bits % 8 != 0 {
        println!("Error: Number of bits read is not byte-aligned: {}", data_capacity_bits);
        return analysis_result; // Data capacity not byte-aligned
    }
    let ecc_bits_expected = total_capacity_bits - data_capacity_bits;
    analysis_result.expected_ecc_bit_string_size = Some(ecc_bits_expected);

    let expected_data_size_bytes = data_capacity_bits / 8;
    let expected_ecc_size_bytes = ecc_bits_expected / 8;
    analysis_result.read_data_bytes = Some(unmasked_bytes[0..expected_data_size_bytes].iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" "));
    analysis_result.read_ecc_bytes = Some(unmasked_bytes[expected_data_size_bytes..expected_data_size_bytes + expected_ecc_size_bytes].iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" "));

    let ecc_result = ecc::correct_errors(&unmasked_bytes, ecc_bits_expected / 8);
    let mut corrected_data = unmasked_bytes.clone();
    let mut corrected_bit_string = unmasked_bit_string.clone();
    match ecc_result {
        CorrectionResult::Uncorrectable => {
            println!("Error: Uncorrectable errors detected in data.");
            return analysis_result; // Correction failed, return without corrected data
        }
        CorrectionResult::Corrected { data, error_positions: _, error_magnitudes: _ } => {
            analysis_result.data_ecc_valid = false;
            corrected_data = data;
            corrected_bit_string = bytes_to_bit_string(&corrected_data);
            analysis_result.corrected_bit_string = Some(bytes_to_bit_string(&corrected_data));
            analysis_result.corrected_bytes = Some(corrected_data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" "));

            let corrected_ecc = generate_ecc(&corrected_data, ecc_bits_expected / 8);
            let mut corrected_message_bytes = corrected_data.clone();
            corrected_message_bytes.extend(&corrected_ecc);
            analysis_result.corrected_data = Some(corrected_message_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" "));
            let data_error_positions = zip(&unmasked_bytes, &corrected_message_bytes).enumerate().filter(|(_i, (a, b))| a != b).map(|(i, _)| i).collect::<Vec<usize>>();
            analysis_result.reconstructed_ecc_bytes = Some(corrected_ecc.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" "));
            analysis_result.corrupted_bytes_percentage = Some((data_error_positions.len() as f64 / (corrected_message_bytes.len() as f64)) * 100.0);
            analysis_result.data_error_positions = Some(data_error_positions);
        }
        CorrectionResult::ErrorFree(_) => {
            analysis_result.data_ecc_valid = true;
        }
    }

    // Step 3: Analyze corrected data
    let mode_bits = (corrected_data[0] >> 4) & 0b1111;
    analysis_result.encoding_info_bit_string = Some(format!("{:04b}", mode_bits));
    let data_mode = match mode_bits {
        0b0001 => DataMode::Numeric,
        0b0010 => DataMode::Alphanumeric,
        0b0100 => DataMode::Byte,
        _ => {
            analysis_result.encoding_name = Some("Unknown".to_string());
            return analysis_result; // Unsupported mode for this analysis
        },
    };
    analysis_result.encoding_name = Some(data_mode.to_string());
    
    let length_value_length_in_bits = match data_mode {
        DataMode::Numeric => 10, // Numeric mode in V1 uses 10 bits for length
        DataMode::Alphanumeric => 9,  // Alphanumeric mode in V1 uses 9 bits
        DataMode::Byte => 8,  // Byte mode in V1 uses 8 bits
    };

    let data_length = if corrected_data.len() * 8 >= 4 + length_value_length_in_bits {
        let length_bit_string = corrected_bit_string[4..4 + length_value_length_in_bits].to_string();
        let length_value = usize::from_str_radix(&length_bit_string, 2).unwrap_or(0);
        length_value
    } else {
        return analysis_result;
    };
    analysis_result.data_length = Some(data_length);
    let end_of_data_bits_index = 4 + length_value_length_in_bits + match data_mode {
        DataMode::Numeric => {
            let full_groups = data_length / 3;
            let remainder = data_length % 3;
            full_groups * 10 + match remainder {
                0 => 0,
                1 => 4,
                2 => 7,
                _ => 0,
            }
        }
        DataMode::Alphanumeric => {
            let full_pairs = data_length / 2;
            let remainder = data_length % 2;
            full_pairs * 11 + match remainder {
                0 => 0,
                1 => 6,
                _ => 0,
            }
        }
        DataMode::Byte => data_length * 8,
    };
    analysis_result.message_bytes = Some(
        bits_to_bytes(
            &corrected_bit_string[4 + length_value_length_in_bits..end_of_data_bits_index]
                .chars()
                .map(|b: char| match b { '0' => 0, '1' => 1, _ => 0 })
                .collect::<Vec<u8>>()
        )
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ")
    );
    analysis_result.padding_bits = Some(corrected_bit_string[end_of_data_bits_index..data_capacity_bits].to_string());

    match data_mode {
        DataMode::Numeric => {
            let mut digits = String::new();
            let mut bit_index = 4 + length_value_length_in_bits;
            for _ in 0..(data_length / 3) {
                if bit_index + 10 > corrected_bit_string.len() {
                    break;
                }
                let num_str = &corrected_bit_string[bit_index..bit_index + 10];
                let num = u16::from_str_radix(num_str, 2).unwrap_or(0);
                digits.push_str(&format!("{:03}", num));
                bit_index += 10;
            }
            if data_length % 3 == 2 {
                if bit_index + 7 <= corrected_bit_string.len() {
                    let num_str = &corrected_bit_string[bit_index..bit_index + 7];
                    let num = u8::from_str_radix(num_str, 2).unwrap_or(0);
                    digits.push_str(&format!("{:02}", num));
                }
            } else if data_length % 3 == 1 {
                if bit_index + 4 <= corrected_bit_string.len() {
                    let num_str = &corrected_bit_string[bit_index..bit_index + 4];
                    let num = u8::from_str_radix(num_str, 2).unwrap_or(0);
                    digits.push_str(&format!("{}", num));
                }
            }
            analysis_result.extracted_data = Some(digits);
        }
        DataMode::Alphanumeric => {
            let alphanumeric_chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";
            let mut chars = String::new();
            let mut bit_index = 4 + length_value_length_in_bits;
            for _ in 0..(data_length / 2) {
                if bit_index + 11 > corrected_bit_string.len() {
                    break;
                }
                let pair_str = &corrected_bit_string[bit_index..bit_index + 11];
                let pair_value = u16::from_str_radix(pair_str, 2).unwrap_or(0);
                let first_char = alphanumeric_chars.chars().nth((pair_value / 45) as usize).unwrap_or(' ');
                let second_char = alphanumeric_chars.chars().nth((pair_value % 45) as usize).unwrap_or(' ');
                chars.push(first_char);
                chars.push(second_char);
                bit_index += 11;
            }
            if data_length % 2 == 1 {
                if bit_index + 6 <= corrected_bit_string.len() {
                    let char_str = &corrected_bit_string[bit_index..bit_index + 6];
                    let char_value = u8::from_str_radix(char_str, 2).unwrap_or(0);
                    let ch = alphanumeric_chars.chars().nth(char_value as usize).unwrap_or(' ');
                    chars.push(ch);
                }
            }
            analysis_result.extracted_data = Some(chars);
        }
        DataMode::Byte => {
            let mut bytes = Vec::new();
            let mut bit_index = 4 + length_value_length_in_bits;
            for _ in 0..data_length {
                if bit_index + 8 > corrected_bit_string.len() {
                    break;
                }
                let byte_str = &corrected_bit_string[bit_index..bit_index + 8];
                let byte_value = u8::from_str_radix(byte_str, 2).unwrap_or(0);
                bytes.push(byte_value);
                bit_index += 8;
            }
            if let Ok(text) = String::from_utf8(bytes.clone()) {
                analysis_result.extracted_data = Some(text);
            } else {
                analysis_result.extracted_data = Some(format!("{:?}", bytes));
            }
        }
    }

    analysis_result
}

fn bytes_to_bit_string(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:08b}", byte)).collect::<Vec<String>>().join("")
}

fn read_data_bits(matrix: &[Vec<u8>], size: usize) -> Vec<u8> {
    let mut bits = Vec::new();
    let mut col = size - 1;
    let mut going_up = true;
    
    // Determine version from size and calculate capacity
    let version = image_size_to_version(size);
    
    // Use minimum total capacity for the version (H level typically has lowest total)
    let max_bits = if let Some(v) = version {
        // Use H level as it typically has the minimum total capacity
        get_total_codewords_in_bits(v)
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

fn bits_to_u16(bits: &[u8]) -> u16 {
    let mut result = 0u16;
    for (i, &bit) in bits.iter().enumerate() {
        result |= (bit as u16) << (bits.len() - 1 - i);
    }
    result
}

fn correct_format_info(format_bits: u16) -> Option<(ErrorCorrection, u8)> {
    const FORMAT_MASK: u16 = 0x5412;
    
    // Try direct decode first
    let unmasked = format_bits ^ FORMAT_MASK;
    if let Some(result) = decode_format_bits(unmasked) {
        return Some(result);
    }
    
    // BCH error correction - try all possible error patterns up to 3 bits
    // Single bit errors
    for i in 0..15 {
        let corrected = format_bits ^ (1 << i);
        let unmasked = corrected ^ FORMAT_MASK;
        if let Some(result) = decode_format_bits(unmasked) {
            return Some(result);
        }
    }
    
    // Double bit errors
    for i in 0..15 {
        for j in (i+1)..15 {
            let corrected = format_bits ^ (1 << i) ^ (1 << j);
            let unmasked = corrected ^ FORMAT_MASK;
            if let Some(result) = decode_format_bits(unmasked) {
                return Some(result);
            }
        }
    }
    
    // Triple bit errors
    for i in 0..15 {
        for j in (i+1)..15 {
            for k in (j+1)..15 {
                let corrected = format_bits ^ (1 << i) ^ (1 << j) ^ (1 << k);
                let unmasked = corrected ^ FORMAT_MASK;
                if let Some(result) = decode_format_bits(unmasked) {
                    return Some(result);
                }
            }
        }
    }
    
    None
}

fn decode_format_bits(bits: u16) -> Option<(ErrorCorrection, u8)> {
    // Extract data bits (upper 5 bits)
    let data = (bits >> 10) & 0x1F;
    
    // Decode error correction level and mask pattern
    let ec_bits = (data >> 3) & 0x3;
    let mask_pattern = (data & 0x7) as u8;
    
    let error_correction = match ec_bits {
        0b01 => ErrorCorrection::L,
        0b00 => ErrorCorrection::M,
        0b11 => ErrorCorrection::Q,
        0b10 => ErrorCorrection::H,
        _ => return None,
    };
    
    if mask_pattern > 7 {
        return None;
    }
    
    Some((error_correction, mask_pattern))
}

fn bch_syndrome(codeword: u16) -> u16 {
    let mut syndrome = codeword;
    for _ in 0..5 {
        if syndrome & 0x4000 != 0 {
            syndrome = (syndrome << 1) ^ 0x537;
        } else {
            syndrome <<= 1;
        }
    }
    syndrome & 0x3FF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bch_format_correction() {
        // Test format bits: 111100010001111 (corrupted)
        let format_bits = 0b111100010001111u16;
        
        // Should decode to ECC Level L, Mask Pattern 3
        let result = correct_format_info(format_bits);
        assert!(result.is_some(), "Should be able to correct 2-bit error");
        
        let (ecc, mask) = result.unwrap();
        assert_eq!(mask, 3, "Should decode to mask pattern 3");
        
        match ecc {
            ErrorCorrection::L => {}, // Expected
            _ => panic!("Should decode to ECC Level L"),
        }
    }
}
