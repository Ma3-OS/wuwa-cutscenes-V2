use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

fn main() {
    let path_str = "D:\\Game\\Wuthering Waves Game\\Client\\Content\\Paks\\pakchunk3-WindowsNoEditor.pak";
    let path = Path::new(path_str);
    
    let mut file = std::fs::File::open(path).unwrap();
    let file_size = file.metadata().unwrap().len();
    
    let footer_size = 221i64;
    file.seek(SeekFrom::End(-footer_size)).unwrap();
    
    let mut footer_data = vec![0u8; footer_size as usize];
    file.read_exact(&mut footer_data).unwrap();
    
    let mut cursor = io::Cursor::new(&footer_data);
    
    let mut guid_bytes = [0u8; 16];
    cursor.read_exact(&mut guid_bytes).unwrap();
    
    let mut b = [0u8; 1];
    cursor.read_exact(&mut b).unwrap();
    let encrypted = b[0] != 0;
    
    let mut magic_bytes = [0u8; 4];
    cursor.read_exact(&mut magic_bytes).unwrap();
    let magic = u32::from_le_bytes(magic_bytes);
    
    let mut ver_bytes = [0u8; 4];
    cursor.read_exact(&mut ver_bytes).unwrap();
    let version = u32::from_le_bytes(ver_bytes);
    
    println!("GUID: {:?}", guid_bytes);
    println!("Encrypted: {}", encrypted);
    println!("Magic: {:#010X} (Expected: {:#010X})", magic, 0x5A6F12E1u32);
    println!("Version: {}", version);
}
