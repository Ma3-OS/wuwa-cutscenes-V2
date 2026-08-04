use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor};

fn main() {
    let path = "D:\\Game\\Wuthering Waves Game\\Client\\Content\\Paks\\pakchunk0optional-WindowsNoEditor.pak";
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => { println!("Failed to open {}: {}", path, e); return; }
    };
    
    let mut footer = vec![0u8; 221];
    file.seek(SeekFrom::End(-221)).unwrap();
    file.read_exact(&mut footer).unwrap();
    
    let index_offset = u64::from_le_bytes(footer[44..52].try_into().unwrap());
    let index_size = u64::from_le_bytes(footer[52..60].try_into().unwrap());
    
    println!("Index offset: {}, size: {}", index_offset, index_size);
    
    let align = if index_size % 16 == 0 { 0 } else { 16 - (index_size % 16) };
    let mut index_data = vec![0u8; (index_size + align) as usize];
    file.seek(SeekFrom::Start(index_offset)).unwrap();
    file.read_exact(&mut index_data).unwrap();
    
    // Decrypt index_data using main key
    let key_hex = "6F80948821CA338739A24D4D9F778BCAC0996B2EF2A73897A789C68AFF05174E";
    let key = hex::decode(key_hex).unwrap();
    use aes::cipher::{BlockCipherDecrypt, KeyInit};
    let cipher = aes::Aes256::new_from_slice(&key).unwrap();
    for chunk in index_data.chunks_mut(16) {
        let mut arr = aes::Block::clone_from_slice(chunk);
        cipher.decrypt_block(&mut arr);
        chunk.copy_from_slice(&arr);
    }
    
    println!("Decrypted index data first 256 bytes:");
    for chunk in index_data.chunks(16).take(16) {
        for b in chunk {
            print!("{:02x} ", b);
        }
        println!();
    }
    
    let mut cur = Cursor::new(&index_data);
    let len = u32::from_le_bytes(cur.get_ref()[0..4].try_into().unwrap()) as i32;
    println!("Mount point len: {}", len);
}
