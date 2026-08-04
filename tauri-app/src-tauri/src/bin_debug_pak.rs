use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use byteorder::{ReadBytesExt, LE};
use tauri_app_lib::pak::parser::parse_pak;

fn main() {
    let pak_dir = "D:\\Game\\Wuthering Waves Game\\Client\\Content\\Paks";
    let pak_name = "pakchunk4-WindowsNoEditor.pak";
    let pak_path = format!("{}/{}", pak_dir, pak_name);
    
    // Fake key
    let key = hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    
    match parse_pak(&pak_path, Some(&key)) {
        Ok(pak_file) => {
            println!("Pak parsed! Entries: {}", pak_file.entries.len());
            for (name, entry) in pak_file.entries.iter().take(2) {
                println!("Entry: {} | offset: {}, size: {}, u_size: {}, method: {}", 
                         name, entry.offset, entry.size, entry.uncompressed_size, entry.compression_method);
            }
        },
        Err(e) => println!("Error: {:?}", e),
    }
}
