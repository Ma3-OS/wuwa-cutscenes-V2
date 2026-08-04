use unpak::Version;
use std::io::{Read, Seek, SeekFrom};
use aes::cipher::{BlockDecrypt, KeyInit};

fn main() {
    let hex = "0x6F80948821CA338739A24D4D9F778BCAC0996B2EF2A73897A789C68AFF05174E";
    let key_bytes = hex::decode(hex.trim_start_matches("0x")).unwrap();
    let mut file = std::fs::File::open("D:\\Game\\Wuthering Waves Game\\Client\\Content\\Paks\\pakchunk3-WindowsNoEditor.pak").unwrap();
    
    // Read footer to get index_offset and index_size
    let mut footer_data = vec![0u8; 221];
    file.seek(SeekFrom::End(-221)).unwrap();
    file.read_exact(&mut footer_data).unwrap();
    let mut cursor = std::io::Cursor::new(&footer_data);
    let mut guid = [0u8; 16];
    cursor.read_exact(&mut guid).unwrap();
    let mut enc = [0u8; 1];
    cursor.read_exact(&mut enc).unwrap();
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic).unwrap();
    let mut ver = [0u8; 4];
    cursor.read_exact(&mut ver).unwrap();
    let mut idx_off = [0u8; 8];
    cursor.read_exact(&mut idx_off).unwrap();
    let index_offset = u64::from_le_bytes(idx_off);
    let mut idx_size = [0u8; 8];
    cursor.read_exact(&mut idx_size).unwrap();
    let index_size = u64::from_le_bytes(idx_size);
    
    println!("Index Offset: {}, Index Size: {}", index_offset, index_size);
    
    // Read and decrypt index
    let align = if index_size % 16 == 0 { 0 } else { 16 - (index_size % 16) };
    let read_size = index_size + align;
    
    let mut index_data = vec![0u8; read_size as usize];
    file.seek(SeekFrom::Start(index_offset)).unwrap();
    file.read_exact(&mut index_data).unwrap();
    
    let key = aes::Aes256::new_from_slice(&key_bytes).unwrap();
    for chunk in index_data.chunks_mut(16) {
        if chunk.len() == 16 {
            let mut arr = aes::cipher::generic_array::GenericArray::clone_from_slice(chunk);
            key.decrypt_block(&mut arr);
            chunk.copy_from_slice(&arr);
        }
    }
    
    println!("Decrypted Index Start:");
    for i in 0..std::cmp::min(128, index_data.len()) {
        print!("{:02X} ", index_data[i]);
    }
    println!();
    
    // Read mount point string length
    let mut index_cursor = std::io::Cursor::new(&index_data);
    let mut len_bytes = [0u8; 4];
    index_cursor.read_exact(&mut len_bytes).unwrap();
    let str_len = i32::from_le_bytes(len_bytes);
    println!("Parsed String Length: {}", str_len);
}
