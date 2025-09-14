use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("gf_tables.rs");

    // Run the Python script to generate GF tables
    let output = Command::new("python3")
        .arg("generate_gf_tables.py")
        .output()
        .expect("Failed to execute generate_gf_tables.py");

    if !output.status.success() {
        panic!("generate_gf_tables.py failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    // Write the generated tables to a Rust file
    let mut f = File::create(&dest_path).unwrap();
    f.write_all(&output.stdout).unwrap();

    println!("cargo:rerun-if-changed=generate_gf_tables.py");
}
