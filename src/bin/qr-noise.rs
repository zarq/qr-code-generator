use image::{Rgb, RgbImage};
use std::env;
use std::process;
use rand::seq::SliceRandom;
use rand::thread_rng;
use qr_generator::pixel_mapping::{get_data_ecc_positions, size_to_version};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        return;
    }
    
    let mut input_file = String::new();
    let mut output_file = String::new();
    let mut percentage = 0.0;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" | "-i" => {
                if i + 1 < args.len() {
                    input_file = add_png_extension(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: --input requires a filename");
                    process::exit(1);
                }
            },
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_file = add_png_extension(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: --output requires a filename");
                    process::exit(1);
                }
            },
            "--percentage" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(p) if p >= 0.0 && p <= 100.0 => percentage = p,
                        _ => {
                            eprintln!("Error: --percentage must be a number between 0 and 100");
                            process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --percentage requires a number");
                    process::exit(1);
                }
            },
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                process::exit(1);
            }
        }
    }
    
    if input_file.is_empty() || output_file.is_empty() || percentage == 0.0 {
        eprintln!("Error: --input, --output, and --percentage are required");
        process::exit(1);
    }
    
    if let Err(e) = add_noise(&input_file, &output_file, percentage) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
    
    println!("Added {:.1}% noise to {} -> {}", percentage, input_file, output_file);
}

fn print_help() {
    println!("qr-noise - Add controlled noise to QR code data areas");
    println!();
    println!("Usage: qr-noise [options]");
    println!();
    println!("Options:");
    println!("  --input, -i <file>       Input PNG file");
    println!("  --output, -o <file>      Output PNG file");
    println!("  --percentage, -p <num>   Percentage of data pixels to flip (0-100)");
    println!("  --help, -h               Show this help message");
}

fn add_png_extension(filename: &str) -> String {
    if filename.ends_with(".png") {
        filename.to_string()
    } else {
        format!("{}.png", filename)
    }
}

fn add_noise(input_file: &str, output_file: &str, percentage: f64) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(input_file)?;
    let mut rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    
    // Detect QR code boundaries and version
    let (qr_start_x, qr_start_y, version, scale) = detect_qr_boundaries(&rgb_img)?;
    
    // Get data/ECC positions using shared module
    let data_positions = get_data_ecc_positions(version);
    
    // Convert QR positions to image pixel coordinates
    let mut image_data_pixels = Vec::new();
    
    for (qr_row, qr_col) in data_positions {
        let img_x = qr_start_x + (qr_col as f32 * scale) as u32;
        let img_y = qr_start_y + (qr_row as f32 * scale) as u32;
        
        // Add all pixels in the scaled block
        let block_size = scale as u32;
        for dy in 0..block_size {
            for dx in 0..block_size {
                let x = img_x + dx;
                let y = img_y + dy;
                if x < width && y < height {
                    image_data_pixels.push((x, y));
                }
            }
        }
    }
    
    // Calculate number of pixels to flip
    let num_to_flip = ((image_data_pixels.len() as f64 * percentage / 100.0).round() as usize)
        .min(image_data_pixels.len());
    
    // Randomly select pixels to flip
    let mut rng = thread_rng();
    let selected_pixels: Vec<_> = image_data_pixels.choose_multiple(&mut rng, num_to_flip).cloned().collect();
    
    // Flip selected pixels
    for (x, y) in selected_pixels {
        let pixel = rgb_img.get_pixel_mut(x, y);
        let is_black = pixel[0] < 128;
        
        if is_black {
            *pixel = Rgb([255, 255, 255]); // Black to white
        } else {
            *pixel = Rgb([0, 0, 0]); // White to black
        }
    }
    
    rgb_img.save(output_file)?;
    Ok(())
}

fn detect_qr_boundaries(img: &RgbImage) -> Result<(u32, u32, qr_generator::types::Version, f32), Box<dyn std::error::Error>> {
    let (width, height) = img.dimensions();
    
    // Find the QR code by looking for the finder pattern (7x7 black square with white border)
    let mut qr_start_x = 0;
    let mut qr_start_y = 0;
    let mut found = false;
    
    // Look for finder pattern: should have significant black area
    'outer: for y in 0..(height - 10) {
        for x in 0..(width - 10) {
            // Check if this looks like a finder pattern
            let mut black_count = 0;
            let mut total_count = 0;
            
            // Check 7x7 area for black pixels
            for dy in 0..7 {
                for dx in 0..7 {
                    if x + dx < width && y + dy < height {
                        let pixel = img.get_pixel(x + dx, y + dy);
                        total_count += 1;
                        if pixel[0] < 128 {
                            black_count += 1;
                        }
                    }
                }
            }
            
            // If more than 60% of the area is black, likely a finder pattern
            if black_count as f32 / total_count as f32 > 0.6 {
                qr_start_x = x;
                qr_start_y = y;
                found = true;
                break 'outer;
            }
        }
    }
    
    if !found {
        return Err("Could not find QR code finder pattern in image".into());
    }
    
    // Find the width by scanning until we hit consistent white border
    let mut qr_pixel_width = 0;
    let mut consecutive_white_cols = 0;
    
    for x in qr_start_x..width {
        // Check if this column is mostly white
        let mut white_count = 0;
        let sample_rows = 10.min(height - qr_start_y);
        
        for dy in 0..sample_rows {
            if qr_start_y + dy < height {
                let pixel = img.get_pixel(x, qr_start_y + dy);
                if pixel[0] >= 200 { // Very white
                    white_count += 1;
                }
            }
        }
        
        // If more than 80% of the column is white, it's likely border
        if white_count as f32 / sample_rows as f32 > 0.8 {
            consecutive_white_cols += 1;
            if consecutive_white_cols >= 3 {
                break; // Found consistent white border
            }
        } else {
            consecutive_white_cols = 0; // Reset counter
            qr_pixel_width = x - qr_start_x + 1;
        }
    }
    
    // Determine QR code logical size and scale
    let mut version = None;
    let mut scale = 0.0;
    
    for size in [21, 25, 29, 33, 37, 41, 45, 49, 53, 57] {
        let test_scale = qr_pixel_width as f32 / size as f32;
        if test_scale >= 1.0 && (test_scale - test_scale.round()).abs() < 0.4 {
            version = size_to_version(size);
            scale = test_scale;
            break;
        }
    }
    
    match version {
        Some(v) => Ok((qr_start_x, qr_start_y, v, scale)),
        None => Err(format!("Could not detect QR code version (width: {})", qr_pixel_width).into()),
    }
}
