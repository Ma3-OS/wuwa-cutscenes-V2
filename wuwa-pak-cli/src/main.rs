use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor};

fn read_u32(cur: &mut Cursor<&[u8]>) -> std::io::Result<u32> {
    let mut buf = [0u8; 4];
    cur.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64(cur: &mut Cursor<&[u8]>) -> std::io::Result<u64> {
    let mut buf = [0u8; 8];
    cur.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn decrypt_bytes(key: &[u8], bytes: &mut [u8]) {
    use aes::cipher::{BlockCipherDecrypt, KeyInit};
    use generic_array::GenericArray;
    if let Ok(cipher) = aes::Aes256::new_from_slice(key) {
        for chunk in bytes.chunks_mut(16) {
            if chunk.len() == 16 {
                let mut arr = *GenericArray::from_slice(chunk);
                cipher.decrypt_block(&mut arr);
                chunk.copy_from_slice(&arr);
            }
        }
    }
}

fn main() {
    // 1. Prepare key
    let key_hex = "0x6F80948821CA338739A24D4D9F778BCAC0996B2EF2A73897A789C68AFF05174E";
    let key_bytes = hex::decode(key_hex.trim_start_matches("0x")).unwrap();

    let pak_path = "D:\\Game\\Wuthering Waves Game\\Client\\Saved\\Resources\\Video\\Paks\\362_0\\Video_362_0-WindowsNoEditor.pak";
    
    let mut file = File::open(pak_path).unwrap();
    let mut footer = vec![0u8; 221];
    file.seek(SeekFrom::End(-221)).unwrap();
    file.read_exact(&mut footer).unwrap();
    
    let mut cur = Cursor::new(footer.as_slice());
    let mut tmp = [0u8; 26];
    cur.read_exact(&mut tmp).unwrap(); // guid, encrypted, magic, ver
    
    let index_offset = read_u64(&mut cur).unwrap();
    let index_size = read_u64(&mut cur).unwrap();
    
    let align = if index_size % 16 == 0 { 0 } else { 16 - (index_size % 16) };
    let mut index_data = vec![0u8; (index_size + align) as usize];
    file.seek(SeekFrom::Start(index_offset)).unwrap();
    file.read_exact(&mut index_data).unwrap();
    
    decrypt_bytes(&key_bytes, &mut index_data);
    
    let mut index_cur = Cursor::new(index_data.as_slice());
    
    // mount point
    let mount_point_len = read_u32(&mut index_cur).unwrap() as i32;
    if mount_point_len > 0 {
        let mut buf = vec![0u8; mount_point_len as usize];
        index_cur.read_exact(&mut buf).unwrap();
    } else if mount_point_len < 0 {
        let count = -mount_point_len as usize;
        let mut buf = vec![0u8; count * 2];
        index_cur.read_exact(&mut buf).unwrap();
    }
    
    let num_entries = read_u32(&mut index_cur).unwrap();
    
    // PathHashIndex (V10+)
    let path_hash_offset = read_u64(&mut index_cur).unwrap();
    let path_hash_size = read_u64(&mut index_cur).unwrap();
    let mut path_hash = [0u8; 20];
    index_cur.read_exact(&mut path_hash).unwrap();
    
    // DirectoryIndex
    let dir_index_offset = read_u64(&mut index_cur).unwrap();
    let dir_index_size = read_u64(&mut index_cur).unwrap();
    let mut dir_hash = [0u8; 20];
    index_cur.read_exact(&mut dir_hash).unwrap();
    
    let align = if dir_index_size % 16 == 0 { 0 } else { 16 - (dir_index_size % 16) };
    let mut dir_data = vec![0u8; (dir_index_size + align) as usize];
    file.seek(SeekFrom::Start(dir_index_offset)).unwrap();
    file.read_exact(&mut dir_data).unwrap();
    
    decrypt_bytes(&key_bytes, &mut dir_data);
    
    // parse directory index
    let mut dcur = Cursor::new(dir_data.as_slice());
    let str_count = read_u32(&mut dcur).unwrap();
    
    for _ in 0..str_count {
        let len = read_u32(&mut dcur).unwrap() as i32;
        if len > 0 {
            let mut buf = vec![0u8; len as usize];
            dcur.read_exact(&mut buf).unwrap();
            let mut name = String::from_utf8_lossy(&buf).to_string();
            name.pop();
            
            let file_count = read_u32(&mut dcur).unwrap();
            for _ in 0..file_count {
                let file_len = read_u32(&mut dcur).unwrap() as i32;
                let mut file_buf = vec![0u8; file_len as usize];
                dcur.read_exact(&mut file_buf).unwrap();
                let mut filename = String::from_utf8_lossy(&file_buf).to_string();
                filename.pop();
                
                let mut hash = [0u8; 20];
                dcur.read_exact(&mut hash).unwrap();
                let bitfield = read_u32(&mut dcur).unwrap();
                
                let is_offset32 = (bitfield & (1 << 31)) != 0;
                let is_uncomp32 = (bitfield & (1 << 30)) != 0;
                let is_size32 = (bitfield & (1 << 29)) != 0;
                let is_enc = (bitfield & (1 << 22)) != 0;
                
                let offset_read = if is_offset32 { read_u32(&mut dcur).unwrap() as u64 } else { read_u64(&mut dcur).unwrap() };
                let uncomp_size_read = if is_uncomp32 { read_u32(&mut dcur).unwrap() as u64 } else { read_u64(&mut dcur).unwrap() };
                
                // My logic in parser.rs swapped them!
                let uncompressed_size = offset_read;
                let offset = uncomp_size_read;
                
                let comp_method = (bitfield >> 23) & 0x3F;
                let size = if comp_method != 0 {
                    if is_size32 { read_u32(&mut dcur).unwrap() as u64 } else { read_u64(&mut dcur).unwrap() }
                } else {
                    uncompressed_size
                };
                
                if filename.contains("M0362_Nvzhu.mp4") {
                    println!("Found {}!", filename);
                    println!("Offset: {}, Size: {}, Uncompressed: {}", offset, size, uncompressed_size);
                    println!("IsEncrypted: {}, CompMethod: {}", is_enc, comp_method);
                    
                    let mut offset_check = [0u8; 8];
                    file.seek(SeekFrom::Start(offset)).unwrap();
                    file.read_exact(&mut offset_check).unwrap();
                    let read_off = u64::from_le_bytes(offset_check);
                    
                    let mut data_offset = offset;
                    if read_off == offset {
                        let mut size_buf = [0u8; 8]; file.read_exact(&mut size_buf).unwrap();
                        let mut cb = [0u8; 4]; file.read_exact(&mut cb).unwrap();
                        let cmethod = u32::from_le_bytes(cb);
                        let mut h = [0u8; 20]; file.read_exact(&mut h).unwrap();
                        if cmethod != 0 {
                            let mut cnt = [0u8; 4]; file.read_exact(&mut cnt).unwrap();
                            let count = u32::from_le_bytes(cnt);
                            file.seek(SeekFrom::Current((count * 16) as i64)).unwrap();
                        }
                        let mut encb = [0u8; 5]; file.read_exact(&mut encb).unwrap();
                        data_offset = file.stream_position().unwrap();
                    }
                    
                    println!("Data Offset: {}", data_offset);
                    
                    let mut bytes_to_read = size as usize;
                    // Apply AES padding just like parser.rs
                    bytes_to_read = (bytes_to_read + 15) & !15;
                    
                    println!("Reading {} bytes...", bytes_to_read);
                    let mut payload = vec![0u8; bytes_to_read];
                    file.seek(SeekFrom::Start(data_offset)).unwrap();
                    file.read_exact(&mut payload).unwrap();
                    
                    // Decrypt
                    decrypt_bytes(&key_bytes, &mut payload);
                    payload.truncate(uncompressed_size as usize);
                    
                    std::fs::write("C:\\Users\\jmach\\projects\\wuwa-cutscenes-main\\test_M0362.mp4", &payload).unwrap();
                    println!("Saved to test_M0362.mp4");
                    return;
                }
            }
        } else if len < 0 {
            // ... skip 
        }
    }
}
