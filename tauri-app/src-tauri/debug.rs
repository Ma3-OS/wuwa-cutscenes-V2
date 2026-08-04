use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

fn main() {
    let pak_path = "D:\\Game\\Wuthering Waves Game\\Client\\Content\\Paks\\pakchunk101-WindowsNoEditor.pak";
    let mut file = match File::open(pak_path) {
        Ok(f) => f,
        Err(e) => { println!("Failed to open: {}", e); return; }
    };
    
    // Read the footer
    let file_len = file.metadata().unwrap().len();
    file.seek(SeekFrom::End(-204)).unwrap(); // Unreal Engine 4 Pak Info size is usually 204 or 45 or 44
    // We can just use the parsing logic from our own parser!
}
