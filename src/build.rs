// build.rs
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo to ONLY rerun this script if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");

    // Mock data mimicking your calculated frequencies/Huffman table
    let frequencies: [u32; 10] = [120, 95, 88, 72, 50, 41, 30, 24, 15, 8];

    // Locate Cargo's isolated output directory
    let out_dir = env::var("OUT_DIR").expect("No OUT_DIR found");
    let dest_path = Path::new(&out_dir).join("huffman_table.rs");
    
    // Open the file buffer
    let file = File::create(&dest_path).expect("Could not create output file");
    let mut writer = BufWriter::new(file);

    // Format and write out the raw array payload as pure Rust syntax
    writeln!(writer, "[")?;
    for freq in frequencies {
        // Using `{:?}` or `{}` formats variables into pure source text
        writeln!(writer, "    {},", freq)?;
    }
    writeln!(writer, "]")?;
}
