use image::{ImageBuffer, Rgb};
use std::env;

mod types;
mod mask;
mod encoding;
use types::{Version, ErrorCorrection, MaskPattern, DataMode, QrConfig};
use mask::apply_mask;
use encoding::{encode_data, EncodedData};

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

fn add_format_info(matrix: &mut Vec<Vec<u8>>, error_correction: ErrorCorrection, mask_pattern: MaskPattern) {
    let format_bits = match error_correction {
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
    let data = (format_bits << 3) | mask_bits;
    
    // Calculate BCH(15,5) error correction for format info
    let mut format_info = data << 10;
    let generator = 0b10100110111; // x^10 + x^8 + x^5 + x^4 + x^2 + x + 1
    
    for _ in 0..5 {
        if (format_info & 0b100000000000000) != 0 {
            format_info ^= generator;
        }
        format_info <<= 1;
    }
    
    let format_info = (data << 10) | (format_info >> 5);
    
    // Top-left format info (around position pattern)
    for i in 0..6 {
        matrix[8][i] = ((format_info >> i) & 1) as u8;
        matrix[i][8] = ((format_info >> i) & 1) as u8;
    }
    matrix[8][7] = ((format_info >> 6) & 1) as u8;
    matrix[8][8] = ((format_info >> 7) & 1) as u8;
    
    // Top-right and bottom-left format info
    let size = matrix.len();
    for i in 0..7 {
        matrix[8][size - 1 - i] = ((format_info >> i) & 1) as u8;
        matrix[size - 1 - i][8] = ((format_info >> i) & 1) as u8;
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

fn apply_format_mask(matrix: &mut Vec<Vec<u8>>) {
    let format_mask = 0b101010000010010; // Fixed format mask pattern
    let size = matrix.len();
    
    // Top-left format info
    for i in 0..6 {
        matrix[8][i] ^= ((format_mask >> i) & 1) as u8;
        matrix[i][8] ^= ((format_mask >> i) & 1) as u8;
    }
    matrix[8][7] ^= ((format_mask >> 6) & 1) as u8;
    matrix[8][8] ^= ((format_mask >> 7) & 1) as u8;
    
    // Top-right and bottom-left format info
    for i in 0..7 {
        matrix[8][size - 1 - i] ^= ((format_mask >> i) & 1) as u8;
        matrix[size - 1 - i][8] ^= ((format_mask >> i) & 1) as u8;
    }
}

fn add_dark_module(matrix: &mut Vec<Vec<u8>>, version: Version) {
    let size = version.size();
    matrix[(4 * version as usize) + 13][8] = 1;
}

fn generate_qr_matrix(url: &str, config: &QrConfig) -> Vec<Vec<u8>> {
    let size = config.version.size();
    let mut matrix = vec![vec![0u8; size]; size];
    
    // Encode the data
    let encoded = encode_data(url, config.version, config.error_correction, config.data_mode);
    
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
    
    if !config.skip_format_mask {
        apply_format_mask(&mut matrix);
    }
    
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
    println!("Usage: {} <output_file.png> <url> [options]", program_name);
    println!();
    println!("Options:");
    println!("  --version, -v [1-7]        QR code version (default: 3)");
    println!("  [mask_pattern]             0-7 (default: 0)");
    println!("  --skip-mask, -s            Skip mask application");
    println!("  --skip-format-mask, -sfm   Skip format mask application");
    println!("  --byte-mode, -b            Use byte mode encoding (default)");
    println!("  --alphanumeric-mode, -a    Use alphanumeric mode encoding");
    println!("  --help, -h                 Show this help message");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Check for help first
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_help(&args[0]);
        return Ok(());
    }
    
    if args.len() < 3 {
        print_help(&args[0]);
        std::process::exit(1);
    }
    
    let filename = &args[1];
    let url = &args[2];
    
    // Add .png extension if not present
    let output_filename = if filename.ends_with(".png") {
        filename.to_string()
    } else {
        format!("{}.png", filename)
    };
    
    let mut config = QrConfig::default();
    let mut i = 3;
    
    // Parse remaining arguments
    while i < args.len() {
        match args[i].as_str() {
            "--skip-mask" | "-s" => config.skip_mask = true,
            "--skip-format-mask" | "-sfm" => config.skip_format_mask = true,
            "--byte-mode" | "-b" => config.data_mode = DataMode::Byte,
            "--alphanumeric-mode" | "-a" => config.data_mode = DataMode::Alphanumeric,
            "--help" | "-h" => {
                print_help(&args[0]);
                return Ok(());
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
                            i += 1; // Skip the version number
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
                if let Ok(pattern) = args[i].parse::<u8>() {
                    config.mask_pattern = match pattern {
                        0 => MaskPattern::Pattern0,
                        1 => MaskPattern::Pattern1,
                        n if n <= 7 => MaskPattern::Pattern0, // Default for unimplemented patterns
                        _ => {
                            eprintln!("Invalid mask pattern. Use 0-7.");
                            std::process::exit(1);
                        }
                    };
                } else {
                    eprintln!("Unknown argument: {}", args[i]);
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }
    
    let matrix = generate_qr_matrix(url, &config);
    matrix_to_png(&matrix, &output_filename)?;
    
    let mask_status = if config.skip_mask { "skipped" } else { "applied" };
    let format_mask_status = if config.skip_format_mask { "skipped" } else { "applied" };
    println!("QR code saved to {} (Version {:?}) with mask pattern {:?} ({}) and format mask ({}) using {:?} mode", 
             output_filename, config.version, config.mask_pattern, mask_status, format_mask_status, config.data_mode);
    Ok(())
}
