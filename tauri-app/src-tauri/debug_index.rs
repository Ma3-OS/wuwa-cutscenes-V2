use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() {
    let pak_path = "D:\\Game\\Wuthering Waves Game\\Client\\Saved\\Resources\\Video\\Paks\\142_2\\Video_142_2-WindowsNoEditor.pak";
    let mut file = File::open(pak_path).unwrap();
    
    file.seek(SeekFrom::End(-204)).unwrap();
    let mut footer = [0u8; 204];
    file.read_exact(&mut footer).unwrap();
    
    let encrypted_index = footer[16] != 0; // enc flag is at offset 16 in 204-byte footer?
    // Wait, in my parse_pak:
    // let mut guid = [0u8; 16]; cur.read_exact(&mut guid)?;
    // let mut enc = [0u8; 1]; cur.read_exact(&mut enc)?;
    let encrypted_index = footer[16] != 0;
    println!("Encrypted index: {}", encrypted_index);
}
