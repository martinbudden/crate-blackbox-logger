// build.rs
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

// Safely pull in your exact types and logic directly from your src tree
#[path = "src/huffman_generate.rs"]
mod huffman_generate;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo to monitor source adjustments accurately
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/huffman_shared.rs");

    // Initialize your input slice matching the requested 256 array slot bounds
    let mut frequencies = [0u32; 256];

    // Example frequency assignments matching your table scale preferences
    frequencies[0x00] = 610104;
    frequencies[0x01] = 192568;
    frequencies[0x02] = 141231;
    frequencies[0x03] = 65118;
    frequencies[0x04] = 71028;
    frequencies[0x05] = 32168;
    frequencies[0x06] = 29934;
    frequencies[0x07] = 20980;
    frequencies[0x08] = 23518;
    frequencies[0x09] = 15709;
    frequencies[0x0A] = 16430;
    frequencies[0x0B] = 12690;
    frequencies[0x0C] = 14099;
    frequencies[0x0D] = 10520;
    frequencies[0x0E] = 10637;
    frequencies[0x0F] = 9438;
    frequencies[0x10] = 33563;
    frequencies[0x11] = 12061;
    frequencies[0x12] = 8866;
    frequencies[0x13] = 7028;
    frequencies[0x14] = 8640;
    frequencies[0x15] = 6559;
    frequencies[0x16] = 6152;
    frequencies[0x17] = 5529;
    frequencies[0x18] = 5546;
    frequencies[0x19] = 4803;
    frequencies[0x1A] = 5164;
    frequencies[0x1B] = 4188;
    frequencies[0x1C] = 4494;
    frequencies[0x1D] = 3784;
    frequencies[0x1E] = 4743;
    frequencies[0x1F] = 8888;
    frequencies[0x20] = 7765;
    frequencies[0x21] = 4472;
    frequencies[0x22] = 3510;
    frequencies[0x23] = 3848;
    frequencies[0x24] = 2666;
    frequencies[0x25] = 2308;
    frequencies[0x26] = 2329;
    frequencies[0x27] = 2024;
    frequencies[0x28] = 2198;
    frequencies[0x29] = 1853;
    frequencies[0x2A] = 1889;
    frequencies[0x2B] = 1706;
    frequencies[0x2C] = 1825;
    frequencies[0x2D] = 1688;
    frequencies[0x2E] = 1896;
    frequencies[0x2F] = 2704;
    frequencies[0x30] = 3569;
    frequencies[0x31] = 1974;
    frequencies[0x32] = 1899;
    frequencies[0x33] = 1544;
    frequencies[0x34] = 1351;
    frequencies[0x35] = 1365;
    frequencies[0x36] = 1312;
    frequencies[0x37] = 1239;
    frequencies[0x38] = 1246;
    frequencies[0x39] = 1177;
    frequencies[0x3A] = 1211;
    frequencies[0x3B] = 1108;
    frequencies[0x3C] = 1227;
    frequencies[0x3D] = 1123;
    frequencies[0x3E] = 1228;
    frequencies[0x3F] = 1421;
    frequencies[0x40] = 28130;
    frequencies[0x41] = 4635;
    frequencies[0x42] = 1175;
    frequencies[0x43] = 1068;
    frequencies[0x44] = 9762;
    frequencies[0x45] = 1000; // E
    frequencies[0x46] = 1036;
    frequencies[0x47] = 1000; // G
    frequencies[0x48] = 1000; // H
    frequencies[0x49] = 1000; // I
    frequencies[0x4A] = 973;
    frequencies[0x4B] = 944;
    frequencies[0x4C] = 889;
    frequencies[0x4D] = 941;
    frequencies[0x4E] = 959;
    frequencies[0x4F] = 945;
    frequencies[0x50] = 1500; // P
    frequencies[0x51] = 1677;
    frequencies[0x52] = 1026;
    frequencies[0x53] = 1000; // S
    frequencies[0x54] = 2411;
    frequencies[0x55] = 1286;
    frequencies[0x56] = 836;
    frequencies[0x57] = 785;
    frequencies[0x58] = 839;
    frequencies[0x59] = 749;
    frequencies[0x5A] = 851;
    frequencies[0x5B] = 794;
    frequencies[0x5C] = 788;
    frequencies[0x5D] = 772;
    frequencies[0x5E] = 760;
    frequencies[0x5F] = 761;
    frequencies[0x60] = 748;
    frequencies[0x61] = 821;
    frequencies[0x62] = 801;
    frequencies[0x63] = 796;
    frequencies[0x64] = 804;
    frequencies[0x65] = 771;
    frequencies[0x66] = 701;
    frequencies[0x67] = 757;
    frequencies[0x68] = 743;
    frequencies[0x69] = 700;
    frequencies[0x6A] = 739;
    frequencies[0x6B] = 722;
    frequencies[0x6C] = 667;
    frequencies[0x6D] = 688;
    frequencies[0x6E] = 645;
    frequencies[0x6F] = 652;
    frequencies[0x70] = 624;
    frequencies[0x71] = 622;
    frequencies[0x72] = 688;
    frequencies[0x73] = 624;
    frequencies[0x74] = 678;
    frequencies[0x75] = 622;
    frequencies[0x76] = 659;
    frequencies[0x77] = 581;
    frequencies[0x78] = 633;
    frequencies[0x79] = 578;
    frequencies[0x7A] = 606;
    frequencies[0x7B] = 601;
    frequencies[0x7C] = 565;
    frequencies[0x7D] = 589;
    frequencies[0x7E] = 530;
    frequencies[0x7F] = 548;
    frequencies[0x80] = 787;
    frequencies[0x81] = 746;
    frequencies[0x82] = 763;
    frequencies[0x83] = 730;
    frequencies[0x84] = 790;
    frequencies[0x85] = 662;
    frequencies[0x86] = 788;
    frequencies[0x87] = 724;
    frequencies[0x88] = 721;
    frequencies[0x89] = 668;
    frequencies[0x8A] = 718;
    frequencies[0x8B] = 667;
    frequencies[0x8C] = 687;
    frequencies[0x8D] = 649;
    frequencies[0x8E] = 784;
    frequencies[0x8F] = 634;
    frequencies[0x90] = 1367;
    frequencies[0x91] = 581;
    frequencies[0x92] = 770;
    frequencies[0x93] = 633;
    frequencies[0x94] = 924;
    frequencies[0x95] = 605;
    frequencies[0x96] = 630;
    frequencies[0x97] = 626;
    frequencies[0x98] = 612;
    frequencies[0x99] = 608;
    frequencies[0x9A] = 645;
    frequencies[0x9B] = 587;
    frequencies[0x9C] = 838;
    frequencies[0x9D] = 578;
    frequencies[0x9E] = 747;
    frequencies[0x9F] = 563;
    frequencies[0xA0] = 1481;
    frequencies[0xA1] = 574;
    frequencies[0xA2] = 840;
    frequencies[0xA3] = 488;
    frequencies[0xA4] = 901;
    frequencies[0xA5] = 494;
    frequencies[0xA6] = 600;
    frequencies[0xA7] = 501;
    frequencies[0xA8] = 605;
    frequencies[0xA9] = 489;
    frequencies[0xAA] = 548;
    frequencies[0xAB] = 475;
    frequencies[0xAC] = 532;
    frequencies[0xAD] = 457;
    frequencies[0xAE] = 521;
    frequencies[0xAF] = 451;
    frequencies[0xB0] = 536;
    frequencies[0xB1] = 451;
    frequencies[0xB2] = 460;
    frequencies[0xB3] = 439;
    frequencies[0xB4] = 425;
    frequencies[0xB5] = 450;
    frequencies[0xB6] = 460;
    frequencies[0xB7] = 399;
    frequencies[0xB8] = 440;
    frequencies[0xB9] = 417;
    frequencies[0xBA] = 431;
    frequencies[0xBB] = 395;
    frequencies[0xBC] = 386;
    frequencies[0xBD] = 366;
    frequencies[0xBE] = 403;
    frequencies[0xBF] = 397;
    frequencies[0xC0] = 458;
    frequencies[0xC1] = 372;
    frequencies[0xC2] = 381;
    frequencies[0xC3] = 351;
    frequencies[0xC4] = 374;
    frequencies[0xC5] = 365;
    frequencies[0xC6] = 352;
    frequencies[0xC7] = 338;
    frequencies[0xC8] = 380;
    frequencies[0xC9] = 324;
    frequencies[0xCA] = 331;
    frequencies[0xCB] = 354;
    frequencies[0xCC] = 344;
    frequencies[0xCD] = 302;
    frequencies[0xCE] = 378;
    frequencies[0xCF] = 345;
    frequencies[0xD0] = 1094;
    frequencies[0xD1] = 623;
    frequencies[0xD2] = 373;
    frequencies[0xD3] = 324;
    frequencies[0xD4] = 315;
    frequencies[0xD5] = 305;
    frequencies[0xD6] = 284;
    frequencies[0xD7] = 299;
    frequencies[0xD8] = 305;
    frequencies[0xD9] = 306;
    frequencies[0xDA] = 326;
    frequencies[0xDB] = 283;
    frequencies[0xDC] = 326;
    frequencies[0xDD] = 287;
    frequencies[0xDE] = 363;
    frequencies[0xDF] = 568;
    frequencies[0xE0] = 4626;
    frequencies[0xE1] = 1492;
    frequencies[0xE2] = 532;
    frequencies[0xE3] = 289;
    frequencies[0xE4] = 285;
    frequencies[0xE5] = 242;
    frequencies[0xE6] = 297;
    frequencies[0xE7] = 254;
    frequencies[0xE8] = 320;
    frequencies[0xE9] = 230;
    frequencies[0xEA] = 284;
    frequencies[0xEB] = 295;
    frequencies[0xEC] = 372;
    frequencies[0xED] = 473;
    frequencies[0xEE] = 492;
    frequencies[0xEF] = 2170;
    frequencies[0xF0] = 19336;
    frequencies[0xF1] = 3407;
    frequencies[0xF2] = 827;
    frequencies[0xF3] = 664;
    frequencies[0xF4] = 286;
    frequencies[0xF5] = 225;
    frequencies[0xF6] = 250;
    frequencies[0xF7] = 202;
    frequencies[0xF8] = 275;
    frequencies[0xF9] = 216;
    frequencies[0xFA] = 240;
    frequencies[0xFB] = 197;
    frequencies[0xFC] = 232;
    frequencies[0xFD] = 307;
    frequencies[0xFE] = 832;
    frequencies[0xFF] = 3190;
    // CALL YOUR EXACT SIGNATURE FUNCTION: No rewritten algorithms
    let huffman_table = huffman_generate::generate_huffman_table(&frequencies);

    // Resolve Cargo targets cleanly via explicit bubble-up operators
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("huffman_table.rs");
    let file = File::create(&dest_path)?;
    let mut writer = BufWriter::new(file);

    // Format the generated file array payload directly into valid structural Rust syntax
    writeln!(writer, "[")?;
    for (ii, &item) in huffman_table.iter().enumerate() {
        //for item in huffman_table.iter() {
        // Formats the code field as hex digits (:x) on a single line
        let display = if (ii as u8).is_ascii_graphic() { ii as u8 as char } else { '.' };
        let value = item.code >> (16 - item.len);
        let len = item.len as usize;
        writeln!(
            writer,
            "    HuffmanCode {{ len: {:2}, code: 0x{:04x} }}, // 0x{:02x} {} 0b_{:0len$b}",
            item.len, item.code, ii, display, value
        )?;
    }
    writeln!(writer, "]")?;

    Ok(())
}
