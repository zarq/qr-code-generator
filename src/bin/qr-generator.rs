use image::{ImageBuffer, Rgb};
use std::env;
use qr_tools::types::{QrConfig, OutputFormat, ErrorCorrection, DataMode, MaskPattern};
use qr_tools::generator::generate_qr_matrix;

fn matrix_to_svg(matrix: &Vec<Vec<u8>>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let size = matrix.len();
    let scale = 10;
    let border = 4 * scale;
    let total_size = size * scale + 2 * border;
    
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        total_size, total_size, total_size, total_size
    );
    
    svg.push_str(&format!(r#"<rect width="{}" height="{}" fill="white"/>"#, total_size, total_size));
    
    for (y, row) in matrix.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 1 {
                let rect_x = border + x * scale;
                let rect_y = border + y * scale;
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" fill="black"/>"#,
                    rect_x, rect_y, scale, scale
                ));
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
    let scale = 10;
    let border = 4 * scale;
    let total_size = size * scale + 2 * border;
    
    let mut img = ImageBuffer::new(total_size as u32, total_size as u32);
    
    for (y, row) in matrix.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            let color = if cell == 1 { [0u8, 0u8, 0u8] } else { [255u8, 255u8, 255u8] };
            
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = border + x * scale + dx;
                    let py = border + y * scale + dy;
                    img.put_pixel(px as u32, py as u32, Rgb(color));
                }
            }
        }
    }
    
    img.save(filename)?;
    Ok(())
}

fn print_help(program_name: &str) {
    println!("Usage: {} [OPTIONS] <text>", program_name);
    println!();
    println!("Generate QR codes from text input");
    println!();
    println!("OPTIONS:");
    println!("  -e, --error-correction LEVEL  Error correction level (L, M, Q, H) [default: M]");
    println!("  -m, --mask PATTERN            Mask pattern (0-7) [default: 0]");
    println!("  -d, --data-mode MODE           Data mode (byte, numeric, alphanumeric) [default: byte]");
    println!("  -o, --output FILE              Output filename [default: qr-code.png]");
    println!("  -f, --format FORMAT            Output format (png, svg) [default: png]");
    println!("  -s, --skip-mask                Skip mask application");
    println!("  -h, --help                     Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("  {} \"Hello, World!\"", program_name);
    println!("  {} -e H -m 3 -o my-qr.svg -f svg \"Hello, World!\"", program_name);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let program_name = &args[0];
    
    if args.len() < 2 {
        print_help(program_name);
        return Ok(());
    }
    
    let mut config = QrConfig::default();
    let mut text = String::new();
    let mut i = 1;
    
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help(program_name);
                return Ok(());
            }
            "-e" | "--error-correction" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --error-correction requires a value");
                    return Ok(());
                }
                config.error_correction = match args[i + 1].to_uppercase().as_str() {
                    "L" => ErrorCorrection::L,
                    "M" => ErrorCorrection::M,
                    "Q" => ErrorCorrection::Q,
                    "H" => ErrorCorrection::H,
                    _ => {
                        eprintln!("Error: Invalid error correction level. Use L, M, Q, or H");
                        return Ok(());
                    }
                };
                i += 2;
            }
            "-m" | "--mask" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --mask requires a value");
                    return Ok(());
                }
                let mask_num: u8 = args[i + 1].parse().map_err(|_| "Invalid mask pattern")?;
                config.mask_pattern = match mask_num {
                    0 => MaskPattern::Pattern0,
                    1 => MaskPattern::Pattern1,
                    2 => MaskPattern::Pattern2,
                    3 => MaskPattern::Pattern3,
                    4 => MaskPattern::Pattern4,
                    5 => MaskPattern::Pattern5,
                    6 => MaskPattern::Pattern6,
                    7 => MaskPattern::Pattern7,
                    _ => {
                        eprintln!("Error: Mask pattern must be 0-7");
                        return Ok(());
                    }
                };
                i += 2;
            }
            "-d" | "--data-mode" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --data-mode requires a value");
                    return Ok(());
                }
                config.data_mode = match args[i + 1].to_lowercase().as_str() {
                    "byte" => DataMode::Byte,
                    "numeric" => DataMode::Numeric,
                    "alphanumeric" => DataMode::Alphanumeric,
                    _ => {
                        eprintln!("Error: Invalid data mode. Use byte, numeric, or alphanumeric");
                        return Ok(());
                    }
                };
                i += 2;
            }
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --output requires a filename");
                    return Ok(());
                }
                config.output_filename = args[i + 1].clone();
                i += 2;
            }
            "-f" | "--format" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --format requires a value");
                    return Ok(());
                }
                config.output_format = match args[i + 1].to_lowercase().as_str() {
                    "png" => OutputFormat::Png,
                    "svg" => OutputFormat::Svg,
                    _ => {
                        eprintln!("Error: Invalid format. Use png or svg");
                        return Ok(());
                    }
                };
                i += 2;
            }
            "-s" | "--skip-mask" => {
                config.skip_mask = true;
                i += 1;
            }
            _ => {
                if args[i].starts_with('-') {
                    eprintln!("Error: Unknown option {}", args[i]);
                    return Ok(());
                }
                text = args[i].clone();
                i += 1;
            }
        }
    }
    
    if text.is_empty() {
        eprintln!("Error: No text provided");
        print_help(program_name);
        return Ok(());
    }
    
    let matrix = generate_qr_matrix(&text, &config);
    save_matrix(&matrix, &config)?;
    
    println!("QR code generated: {}", config.output_filename);
    Ok(())
}
