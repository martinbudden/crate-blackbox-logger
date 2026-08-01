// build.rs
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    // 1. Tell Cargo to ONLY rerun this script if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");

    // Mock data mimicking your calculated frequencies/Huffman table
    let frequencies: [u32; 10] = [120, 95, 88, 72, 50, 41, 30, 24, 15, 8];

    // 2. Locate Cargo's isolated output directory
    let out_dir = env::var("OUT_DIR").expect("No OUT_DIR found");
    let dest_path = Path::new(&out_dir).join("huffman_table.rs");
    
    // 3. Open the file buffer
    let file = File::create(&dest_path).expect("Could not create output file");
    let mut writer = BufWriter::new(file);

    // 4. Format and write out the raw array payload as pure Rust syntax
    writeln!(writer, "[").unwrap();
    for freq in frequencies {
        // Using `{:?}` or `{}` formats variables into pure source text
        writeln!(writer, "    {},", freq).unwrap();
    }
    writeln!(writer, "]").unwrap();
}
