use image::{Rgb, RgbImage};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 4 {
        print_help();
        process::exit(1);
    }
    
    let input1 = add_png_extension(&args[1]);
    let input2 = add_png_extension(&args[2]);
    let output = add_png_extension(&args[3]);
    
    if let Err(e) = create_diff(&input1, &input2, &output) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
    
    println!("Diff created: {} vs {} -> {}", input1, input2, output);
}

fn print_help() {
    println!("qr-diff - Compare two PNG files and highlight differences");
    println!();
    println!("Usage: qr-diff <input1.png> <input2.png> <output.png>");
    println!();
    println!("Color coding:");
    println!("  Black/White: Same in both images");
    println!("  Green: White in first, black in second");
    println!("  Red: Black in first, white in second");
}

fn add_png_extension(filename: &str) -> String {
    if filename.ends_with(".png") {
        filename.to_string()
    } else {
        format!("{}.png", filename)
    }
}

fn create_diff(input1: &str, input2: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let img1 = image::open(input1)?.to_rgb8();
    let img2 = image::open(input2)?.to_rgb8();
    
    let (width1, height1) = img1.dimensions();
    let (width2, height2) = img2.dimensions();
    
    if width1 != width2 || height1 != height2 {
        return Err(format!("Images have different dimensions: {}x{} vs {}x{}", 
                          width1, height1, width2, height2).into());
    }
    
    let mut diff_img = RgbImage::new(width1, height1);
    
    for y in 0..height1 {
        for x in 0..width1 {
            let pixel1 = img1.get_pixel(x, y);
            let pixel2 = img2.get_pixel(x, y);
            
            let is_black1 = pixel1[0] < 128;
            let is_black2 = pixel2[0] < 128;
            
            let diff_pixel = match (is_black1, is_black2) {
                (true, true) => Rgb([0, 0, 0]),       // Both black -> black
                (false, false) => Rgb([255, 255, 255]), // Both white -> white
                (false, true) => Rgb([0, 255, 0]),     // White->Black -> green
                (true, false) => Rgb([255, 0, 0]),     // Black->White -> red
            };
            
            diff_img.put_pixel(x, y, diff_pixel);
        }
    }
    
    diff_img.save(output)?;
    Ok(())
}
