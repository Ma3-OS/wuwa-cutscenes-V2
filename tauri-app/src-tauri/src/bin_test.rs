use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use byteorder::{ReadBytesExt, LE};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Let's test with a known pak and offset.
    // Wait, I need an offset! How do I get an offset?
    println!("I need to parse the index first to get an offset...");
    Ok(())
}
