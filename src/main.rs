use image::{ImageBuffer, Rgb};
use std::env;

mod types;
mod mask;
mod encoding;
mod ecc;
use types::{Version, ErrorCorrection, MaskPattern, DataMode, QrConfig};
use mask::apply_mask;
use encoding::{encode_data, EncodedData};
use ecc::generate_ecc;

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

fn place_data_bits(matrix: &mut Vec<Vec<u8>>, encoded: &EncodedData) {
    // Combine data and ECC bits
    let mut all_bits = encoded.data_bits.clone();
    all_bits.extend(&encoded.ecc_bits);
    
    let size = matrix.len();
    let mut bit_index = 0;
    let mut up = true;
    
    // Start from bottom-right, move in zigzag pattern
    let mut col = size - 1;
    while col > 0 && bit_index < all_bits.len() {
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
                if !is_function_module(x, y, size) && bit_index < all_bits.len() {
                    matrix[y][x] = all_bits[bit_index];
                    bit_index += 1;
                }
            }
        }
        
        up = !up;
        col = col.saturating_sub(2);
    }
}

fn is_function_module(x: usize, y: usize, size: usize) -> bool {
    // Position patterns and separators
    if (x < 9 && y < 9) || (x >= size - 8 && y < 9) || (x < 9 && y >= size - 8) {
        return true;
    }
    
    // Timing patterns
    if x == 6 || y == 6 {
        return true;
    }
    
    // Alignment pattern (for Version 3)
    if size > 25 && x >= size - 9 && x < size - 4 && y >= size - 9 && y < size - 4 {
        return true;
    }
    
    // Format info areas
    if (x < 9 && y == 8) || (y < 9 && x == 8) || 
       (x >= size - 8 && y == 8) || (y >= size - 7 && x == 8) {
        return true;
    }
    
    false
}


fn add_dark_module(matrix: &mut Vec<Vec<u8>>, version: Version) {
    let size = version.size();
    matrix[size - 7][8] = 1;
}

fn print_verbose_info(config: &QrConfig, encoded: &EncodedData) {
    println!("=== QR Code Metadata ===");
    println!("Version: {:?} ({}x{})", config.version, config.version.size(), config.version.size());
    println!("Error Correction: {:?}", config.error_correction);
    println!("Data Mode: {:?}", config.data_mode);
    println!("Data Length: {} characters", config.url.len());
    println!("Mask Pattern: {:?}", config.mask_pattern);
    
    println!("\n=== Format Information ===");
    let format_info = get_format_info(config.error_correction, config.mask_pattern);
    println!("Format info with ECC: {:015b}", format_info);
    
    println!("\n=== Data and ECC ===");
    println!("Data bits ({} bits): {:?}", encoded.data_bits.len(), 
             encoded.data_bits.iter().map(|&b| b.to_string()).collect::<Vec<_>>().join(""));
    println!("ECC bits ({} bits): {:?}", encoded.ecc_bits.len(),
             encoded.ecc_bits.iter().map(|&b| b.to_string()).collect::<Vec<_>>().join(""));
    println!();
}

fn generate_qr_matrix(url: &str, config: &QrConfig) -> Vec<Vec<u8>> {
    let size = config.version.size();
    let mut matrix = vec![vec![0u8; size]; size];
    
    // Encode the data
    let encoded = encode_data(url, config.version, config.error_correction, config.data_mode);
    
    if config.verbose {
        print_verbose_info(config, &encoded);
    }
    
    // Place the encoded data
    place_data_bits(&mut matrix, &encoded);
    
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
    add_dark_module(&mut matrix, config.version);
    
    matrix
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
    println!("  --output, -o <file>        Output PNG file (default: qr-code.png)");
    println!("  --url, -u <url>            URL to encode (default: https://www.example.com/)");
    println!("  --version, -v [1-7]        QR code version (default: 3)");
    println!("  --ecc-level, -l [L|M|Q|H]  Error correction level (default: M)");
    println!("  --mask-pattern, -mp [0-7]  Mask pattern (default: 0)");
    println!("  --skip-mask, -s            Skip mask application");
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
    let mut i = 1;
    
    // Parse arguments
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    let filename = &args[i + 1];
                    config.output_filename = if filename.ends_with(".png") {
                        filename.to_string()
                    } else {
                        format!("{}.png", filename)
                    };
                    i += 1;
                } else {
                    eprintln!("Output option requires a filename.");
                    std::process::exit(1);
                }
            }
            "--url" | "-u" => {
                if i + 1 < args.len() {
                    config.url = args[i + 1].clone();
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
            "--version" | "-v" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u8>() {
                        Ok(v @ 1..=7) => {
                            config.version = match v {
                                1 => Version::V1,
                                2 => Version::V2,
                                3 => Version::V3,
                                4 => Version::V4,
                                5 => Version::V5,
                                6 => Version::V6,
                                7 => Version::V7,
                                _ => unreachable!(),
                            };
                            i += 1;
                        }
                        _ => {
                            eprintln!("Invalid version. Use 1-7.");
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Version option requires a value.");
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
    
    // Apply parsed values or use defaults
    let matrix = generate_qr_matrix(&config.url, &config);
    matrix_to_png(&matrix, &config.output_filename)?;
    
    let mask_status = if config.skip_mask { "skipped" } else { "applied" };
    println!("QR code saved to {} (Version {:?}) with mask pattern {:?} ({}) using {:?} mode", 
             config.output_filename, config.version, config.mask_pattern, mask_status, config.data_mode);
    Ok(())
}
