use image::{ImageBuffer, Rgb};
use qr_tools::capacity::get_unencoded_capacity_in_bytes;
use std::env;

use qr_tools::types;
use qr_tools::mask;
use qr_tools::encoding;
use qr_tools::alignment;
use types::{Version, ErrorCorrection, MaskPattern, DataMode, QrConfig, OutputFormat};
use mask::apply_mask;
use encoding::{encode_data, EncodedData};
use alignment::{is_alignment_pattern, get_alignment_positions};

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
            let is_border = dy == 0 || dy == 4 || dx == 0 || dx == 4;
            let is_center = dy == 2 && dx == 2;
            matrix[y + dy][x + dx] = if is_border || is_center { 1 } else { 0 };
        }
    }
}

fn add_timing_patterns(matrix: &mut Vec<Vec<u8>>, size: usize) {
    for i in 8..(size - 8) {
        matrix[6][i] = ((i + 1) % 2) as u8;
        matrix[i][6] = ((i + 1) % 2) as u8;
    }
}

fn get_format_info(error_correction: ErrorCorrection, mask_pattern: MaskPattern) -> u16 {
    // Format info lookup table from https://www.thonky.com/qr-code-tutorial/format-version-tables
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
    
    // Top-left format info (around position pattern)
    // Horizontal: bits 14-9 (leftmost 6 bits)
    for i in 0..6 {
        matrix[8][i] = ((format_info >> (14 - i)) & 1) as u8;
    }
    // Vertical: bits 5-0 (rightmost 6 bits)
    for i in 0..6 {
        matrix[i][8] = ((format_info >> i) & 1) as u8;
    }
    // Special positions
    matrix[8][7] = ((format_info >> 8) & 1) as u8;  // bit 8
    matrix[8][8] = ((format_info >> 7) & 1) as u8;  // bit 7
    matrix[7][8] = ((format_info >> 6) & 1) as u8;  // bit 6
    
    // Top-right and bottom-left format info
    let size = matrix.len();
    for i in 0..8 {
        matrix[8][size - 1 - i] = ((format_info >> i) & 1) as u8;
        if i < 7 {
            matrix[size - 1 - i][8] = ((format_info >> (14 - i)) & 1) as u8;
        }
    }
}

fn place_data_bits(matrix: &mut Vec<Vec<u8>>, encoded: &EncodedData, version: Version) {
    // Get block structure and interleave according to QR spec
    let (data_blocks, ecc_blocks) = get_block_structure(&encoded.data_bits, &encoded.ecc_bits);
    
    // Create interleaved bit stream
    let mut bit_stream = Vec::new();
    
    // Interleave data blocks byte by byte
    let max_data_bytes = data_blocks.iter().map(|b| b.len()).max().unwrap_or(0);
    for byte_index in 0..max_data_bytes {
        for block in &data_blocks {
            if byte_index < block.len() {
                // Convert byte to bits
                for bit_pos in 0..8 {
                    bit_stream.push((block[byte_index] >> (7 - bit_pos)) & 1);
                }
            }
        }
    }
    
    // Interleave ECC blocks byte by byte
    let max_ecc_bytes = ecc_blocks.iter().map(|b| b.len()).max().unwrap_or(0);
    for byte_index in 0..max_ecc_bytes {
        for block in &ecc_blocks {
            if byte_index < block.len() {
                // Convert byte to bits
                for bit_pos in 0..8 {
                    bit_stream.push((block[byte_index] >> (7 - bit_pos)) & 1);
                }
            }
        }
    }
    
    // Place bits in zigzag pattern
    let size = matrix.len();
    let mut bit_index = 0;
    let mut up = true;
    
    // Start from bottom-right, move left in 2-column strips
    let mut col = size - 1;
    while col > 0 && bit_index < bit_stream.len() {
        // Skip timing column
        if col == 6 {
            col -= 1;
        }
        
        // Process two columns at a time
        for row_offset in 0..size {
            let y = if up { size - 1 - row_offset } else { row_offset };
            
            // Right column first, then left column
            for dx in 0..2 {
                let x = col - dx;
                if !is_function_module(x, y, size, version) && bit_index < bit_stream.len() {
                    matrix[y][x] = bit_stream[bit_index];
                    bit_index += 1;
                }
            }
        }
        
        up = !up;
        col = col.saturating_sub(2);
    }
}

fn get_block_structure(data_bits: &[u8], ecc_bits: &[u8]) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    // Convert bits to bytes
    let data_bytes = bits_to_bytes(data_bits);
    let ecc_bytes = bits_to_bytes(ecc_bits);
    
    // For now, simple single block structure
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
    // Position patterns and separators
    if (x < 9 && y < 9) || (x >= size - 8 && y < 9) || (x < 9 && y >= size - 8) {
        return true;
    }
    
    // Timing patterns
    if x == 6 || y == 6 {
        return true;
    }
    
    // Alignment patterns
    if is_alignment_pattern(x, y, version) {
        return true;
    }
    
    // Format info areas
    if (x < 9 && y == 8) || (y < 9 && x == 8) || 
       (x >= size - 8 && y == 8) || (y >= size - 7 && x == 8) {
        return true;
    }
    
    // Version info areas (for V7+)
    if size >= 45 { // V7+ (V7=45, V8=49, V9=53, V10=57) 
        if (x < 6 && y >= size - 11 && y < size - 8) ||
           (y < 6 && x >= size - 11 && x < size - 8) {
            return true;
        }
    }
    
    // Dark module (at position (size-8, 8))
    if x == 8 && y == size - 8 {
        return true;
    }
    
    false
}


fn get_version_info(version: Version) -> Option<u32> {
    let version_num = version as u8;
    if version_num < 7 { return None; }
    
    // BCH(18,6) encoding for version information
    let data = version_num as u32;
    let generator = 0x1f25; // BCH generator polynomial
    let mut result = data << 12;
    
    for _ in 0..6 {
        if result & (1 << 17) != 0 {
            result ^= generator << 5;
        }
        result <<= 1;
    }
    
    Some((data << 12) | (result >> 6))
}

fn add_version_info(matrix: &mut Vec<Vec<u8>>, version: Version) {
    if let Some(version_info) = get_version_info(version) {
        let size = matrix.len();
        
        // Bottom-left version info (6x3 area)
        for i in 0..6 {
            for j in 0..3 {
                let bit = (version_info >> (i * 3 + j)) & 1;
                matrix[size - 11 + j][i] = bit as u8;
            }
        }
        
        // Top-right version info (3x6 area) 
        for i in 0..6 {
            for j in 0..3 {
                let bit = (version_info >> (i * 3 + j)) & 1;
                matrix[i][size - 11 + j] = bit as u8;
            }
        }
    }
}

fn add_alignment_patterns(matrix: &mut Vec<Vec<u8>>, version: Version) {
    let positions = get_alignment_positions(version);
    let size = matrix.len();
    
    for &center_x in &positions {
        for &center_y in &positions {
            // Skip if overlaps with finder patterns
            if (center_x <= 8 && center_y <= 8) ||
               (center_x <= 8 && center_y >= size - 9) ||
               (center_x >= size - 9 && center_y <= 8) {
                continue;
            }
            
            // Place 5x5 alignment pattern
            for dy in 0..5 {
                for dx in 0..5 {
                    let x = center_x - 2 + dx;
                    let y = center_y - 2 + dy;
                    
                    let is_dark = (dx == 0 || dx == 4 || dy == 0 || dy == 4) || (dx == 2 && dy == 2);
                    matrix[y][x] = if is_dark { 1 } else { 0 };
                }
            }
        }
    }
}

fn add_dark_module(matrix: &mut Vec<Vec<u8>>, _version: Version) {
    let size = matrix.len();
    let dark_module_pos = size - 8;
    matrix[dark_module_pos][8] = 1;
}

fn print_verbose_info(config: &QrConfig, encoded: &EncodedData, version: Version) {
    println!("=== QR Code Metadata ===");
    println!("Version: {:?} ({}x{}) - Auto-calculated", version, version.size(), version.size());
    println!("Error Correction: {:?}", config.error_correction);
    println!("Data Mode: {:?}", config.data_mode);
    println!("Data Length: {} characters", config.data.len());
    println!("Mask Pattern: {:?}", config.mask_pattern);
    
    println!("\n=== Format Information ===");
    let format_info = get_format_info(config.error_correction, config.mask_pattern);
    println!("Format info with ECC: {:015b}", format_info);
    
    println!("\n=== Data and ECC Blocks ===");
    let (data_blocks, ecc_blocks) = get_block_structure(&encoded.data_bits, &encoded.ecc_bits);
    
    for (i, block) in data_blocks.iter().enumerate() {
        let hex_bytes: Vec<String> = block.iter().map(|b| format!("{:02X}", b)).collect();
        println!("Data Block {}: {} bytes", i + 1, block.len());
        println!("  Hex: {}", hex_bytes.join(" "));
    }
    
    for (i, block) in ecc_blocks.iter().enumerate() {
        let hex_bytes: Vec<String> = block.iter().map(|b| format!("{:02X}", b)).collect();
        println!("ECC Block {}: {} bytes", i + 1, block.len());
        println!("  Hex: {}", hex_bytes.join(" "));
    }
    
    println!("\n=== Raw Bit Streams ===");
    println!("Data bits ({} bits): {}", encoded.data_bits.len(), 
             format_bits_with_spaces(&encoded.data_bits));
    println!("ECC bits ({} bits): {}", encoded.ecc_bits.len(),
             format_bits_with_spaces(&encoded.ecc_bits));
    println!();
}

fn format_bits_with_spaces(bits: &[u8]) -> String {
    bits.chunks(8)
        .map(|chunk| chunk.iter().map(|&b| b.to_string()).collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

fn calculate_version(data: &str, error_correction: ErrorCorrection, data_mode: DataMode) -> Version {
    let data_length = data.len();
    
    // Find minimum version that can hold the data
    for version_num in 1..=40 {
        let version = Version::from_u8(version_num).unwrap_or(Version::V40);
        let capacity = get_unencoded_capacity_in_bytes(version, error_correction, data_mode);
        if data_length <= capacity {
            return version;
        }
    }
    
    panic!("Data too large: {} characters cannot fit in any QR code version with {:?} error correction and {:?} mode", 
           data_length, error_correction, data_mode);
}



fn generate_qr_matrix(data: &str, config: &QrConfig) -> Vec<Vec<u8>> {
    // Calculate appropriate version based on data
    let version = calculate_version(data, config.error_correction, config.data_mode);
    let size = version.size();
    let mut matrix = vec![vec![0u8; size]; size];
    
    // Encode the data
    let encoded = encode_data(data, version, config.error_correction, config.data_mode);
    
    if config.verbose {
        print_verbose_info(config, &encoded, version);
    }
    
    // Place the encoded data
    place_data_bits(&mut matrix, &encoded, version);
    
    if !config.skip_mask {
        apply_mask(&mut matrix, config.mask_pattern);
    }
    
    add_position_pattern(&mut matrix, 0, 0);
    add_position_pattern(&mut matrix, size - 7, 0);
    add_position_pattern(&mut matrix, 0, size - 7);
    
    if size > 21 {
        add_alignment_pattern(&mut matrix, size - 9, size - 9);
    }
    
    add_timing_patterns(&mut matrix, size);
    add_format_info(&mut matrix, config.error_correction, config.mask_pattern);
    add_version_info(&mut matrix, version);
    add_alignment_patterns(&mut matrix, version);
    add_dark_module(&mut matrix, version);
    
    matrix
}

fn matrix_to_svg(matrix: &Vec<Vec<u8>>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let size = matrix.len();
    let scale = 10;
    let svg_size = size * scale;
    
    let mut svg = String::new();
    svg.push_str(&format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">
<rect width="100%" height="100%" fill="white"/>
"#, svg_size, svg_size, svg_size, svg_size));
    
    for (y, row) in matrix.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            if pixel == 1 {
                let px = x * scale;
                let py = y * scale;
                svg.push_str(&format!(r#"<rect x="{}" y="{}" width="{}" height="{}" fill="black"/>
"#, px, py, scale, scale));
            }
        }
    }
    
    svg.push_str("</svg>");
    std::fs::write(filename, svg)?;
    Ok(())
}

fn save_matrix(matrix: &Vec<Vec<u8>>, config: &QrConfig) -> Result<(), Box<dyn std::error::Error>> {
    match config.output_format {
        OutputFormat::Png => matrix_to_png(matrix, &config.output_filename),
        OutputFormat::Svg => matrix_to_svg(matrix, &config.output_filename),
    }
}

fn matrix_to_png(matrix: &Vec<Vec<u8>>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let size = matrix.len();
    let img_size = size + 4;
    let img = ImageBuffer::from_fn(img_size as u32, img_size as u32, |x, y| {
        if x < 2 || x >= (size + 2) as u32 || y < 2 || y >= (size + 2) as u32 {
            Rgb([255u8, 255u8, 255u8])
        } else {
            let pixel_value = if matrix[(y - 2) as usize][(x - 2) as usize] == 0 { 255u8 } else { 0u8 };
            Rgb([pixel_value, pixel_value, pixel_value])
        }
    });
    
    img.save(filename)?;
    Ok(())
}

fn print_help(program_name: &str) {
    println!("Usage: {} [options]", program_name);
    println!();
    println!("Options:");
    println!("  --output, -o <file>        Output file (default: qr-code.png)");
    println!("  --png, -P                  Output PNG format (default)");
    println!("  --svg, -S                  Output SVG format");
    println!("  --data, -d <data>            Data to encode (default: https://www.example.com/)");
    println!("  --ecc-level, -l [L|M|Q|H]  Error correction level (default: M)");
    println!("  --mask-pattern, -mp [0-7]  Mask pattern (default: 0)");
    println!("  --skip-mask, -s            Skip mask application");
    println!("  --numeric, -n              Use numeric mode encoding");
    println!("  --byte-mode, -b            Use byte mode encoding (default)");
    println!("  --alphanumeric-mode, -a    Use alphanumeric mode encoding");
    println!("  --verbose, -V              Print detailed QR code information
  --help, -h                 Show this help message");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Check for help first
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_help(&args[0]);
        return Ok(());
    }
    
    let mut config = QrConfig::default();
    let mut png_explicitly_set = false;
    let mut svg_explicitly_set = false;
    let mut i = 1;
    
    // Parse arguments
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    config.output_filename = args[i + 1].clone();
                    i += 1;
                } else {
                    eprintln!("Output option requires a filename.");
                    std::process::exit(1);
                }
            }
            "--png" | "-P" => {
                if svg_explicitly_set {
                    eprintln!("Error: --png and --svg are mutually exclusive.");
                    std::process::exit(1);
                }
                config.output_format = OutputFormat::Png;
                png_explicitly_set = true;
            }
            "--svg" | "-S" => {
                if png_explicitly_set {
                    eprintln!("Error: --png and --svg are mutually exclusive.");
                    std::process::exit(1);
                }
                config.output_format = OutputFormat::Svg;
                svg_explicitly_set = true;
            }
            "--data" | "-d" => {
                if i + 1 < args.len() {
                    config.data = args[i + 1].clone();
                    i += 1;
                } else {
                    eprintln!("URL option requires a value.");
                    std::process::exit(1);
                }
            }
            "--mask-pattern" | "-mp" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u8>() {
                        Ok(0) => config.mask_pattern = MaskPattern::Pattern0,
                        Ok(1) => config.mask_pattern = MaskPattern::Pattern1,
                        Ok(2) => config.mask_pattern = MaskPattern::Pattern2,
                        Ok(3) => config.mask_pattern = MaskPattern::Pattern3,
                        Ok(4) => config.mask_pattern = MaskPattern::Pattern4,
                        Ok(5) => config.mask_pattern = MaskPattern::Pattern5,
                        Ok(6) => config.mask_pattern = MaskPattern::Pattern6,
                        Ok(7) => config.mask_pattern = MaskPattern::Pattern7,
                        _ => {
                            eprintln!("Invalid mask pattern. Use 0-7.");
                            std::process::exit(1);
                        }
                    }
                    i += 1;
                } else {
                    eprintln!("Mask pattern option requires a value.");
                    std::process::exit(1);
                }
            }
            "--skip-mask" | "-s" => config.skip_mask = true,
            "--numeric" | "-n" => config.data_mode = DataMode::Numeric,
            "--byte-mode" | "-b" => config.data_mode = DataMode::Byte,
            "--alphanumeric-mode" | "-a" => config.data_mode = DataMode::Alphanumeric,
            "--verbose" | "-V" => config.verbose = true,
            "--help" | "-h" => {
                print_help(&args[0]);
                return Ok(());
            }
            "--ecc-level" | "-l" => {
                if i + 1 < args.len() {
                    match args[i + 1].as_str() {
                        "L" => config.error_correction = ErrorCorrection::L,
                        "M" => config.error_correction = ErrorCorrection::M,
                        "Q" => config.error_correction = ErrorCorrection::Q,
                        "H" => config.error_correction = ErrorCorrection::H,
                        _ => {
                            eprintln!("Invalid ECC level. Use L, M, Q, or H.");
                            std::process::exit(1);
                        }
                    }
                    i += 1;
                } else {
                    eprintln!("ECC level option requires a value.");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown argument: {}. Use --help for usage information.", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }
    
    // Handle filename extensions based on output format
    match config.output_format {
        OutputFormat::Png => {
            if config.output_filename == "qr-code.png" {
                // Default case, already correct
            } else if !config.output_filename.ends_with(".png") {
                config.output_filename = format!("{}.png", config.output_filename);
            }
        }
        OutputFormat::Svg => {
            if config.output_filename == "qr-code.png" {
                config.output_filename = "qr-code.svg".to_string();
            } else if !config.output_filename.ends_with(".svg") {
                config.output_filename = format!("{}.svg", config.output_filename);
            }
        }
    }
    
    // Apply parsed values or use defaults
    let matrix = generate_qr_matrix(&config.data, &config);
    let version = calculate_version(&config.data, config.error_correction, config.data_mode);
    save_matrix(&matrix, &config)?;
    
    let mask_status = if config.skip_mask { "skipped" } else { "applied" };
    println!("QR code saved to {} (Version {:?}) with mask pattern {:?} ({}) using {:?} mode", 
             config.output_filename, version, config.mask_pattern, mask_status, config.data_mode);
    Ok(())
}
