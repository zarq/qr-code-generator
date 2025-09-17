use image::Rgb;
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
    let rgb_img = img.to_rgb8();
    let (img_width, img_height) = rgb_img.dimensions();
    
    // Detect QR code size (assuming 2-pixel border)
    let qr_size = (img_width - 4) as usize; // Remove 2-pixel border on each side
    let version = size_to_version(qr_size).ok_or("Unsupported QR code size")?;
    
    // Get data positions from shared module
    let data_positions = get_data_ecc_positions(version);
    
    // Convert to image coordinates (add 2-pixel border offset)
    let mut image_data_pixels = Vec::new();
    for (qr_row, qr_col) in data_positions {
        let img_x = (qr_col + 2) as u32; // Add border offset
        let img_y = (qr_row + 2) as u32; // Add border offset
        
        if img_x < img_width && img_y < img_height {
            image_data_pixels.push((img_x, img_y));
        }
    }
    
    // Calculate number of pixels to flip
    let num_to_flip = ((image_data_pixels.len() as f64 * percentage / 100.0).round() as usize)
        .min(image_data_pixels.len());
    
    // Randomly select pixels to flip
    let mut rng = thread_rng();
    let selected_pixels: Vec<_> = image_data_pixels.choose_multiple(&mut rng, num_to_flip).cloned().collect();
    
    // Flip selected pixels
    let mut output_img = rgb_img.clone();
    for (x, y) in selected_pixels {
        let pixel = output_img.get_pixel_mut(x, y);
        let is_black = pixel[0] < 128;
        
        if is_black {
            *pixel = Rgb([255, 255, 255]); // Black to white
        } else {
            *pixel = Rgb([0, 0, 0]); // White to black
        }
    }
    
    output_img.save(output_file)?;
    Ok(())
}
