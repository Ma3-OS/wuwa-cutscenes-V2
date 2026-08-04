use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() {
    let path = "D:\\Game\\Wuthering Waves Game\\Client\\Content\\Paks\\pakchunk101-WindowsNoEditor.pak";
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { println!("Failed to open {}: {}", path, e); return; }
    };
    let mut footer = vec![0u8; 221];
    file.seek(SeekFrom::End(-221)).unwrap();
    file.read_exact(&mut footer).unwrap();
    
    let mut guid = [0u8; 16];
    guid.copy_from_slice(&footer[0..16]);
    print!("pakchunk101 GUID: ");
    for b in guid.iter() {
        print!("{:02X}", b);
    }
    println!();
}
