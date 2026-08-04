use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor};
use aes::cipher::{BlockDecrypt, KeyInit};

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

fn read_string(cur: &mut Cursor<&[u8]>) -> std::io::Result<String> {
    let len = read_u32(cur)? as i32;
    if len == 0 { return Ok(String::new()); }
    if len > 0 {
        if len > 1_000_000 {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "string too long"));
        }
        let mut buf = vec![0u8; len as usize];
        cur.read_exact(&mut buf)?;
        buf.pop(); // remove null
        Ok(String::from_utf8_lossy(&buf).to_string())
    } else {
        let count = -len as usize;
        if count > 1_000_000 {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "utf16 string too long"));
        }
        let mut buf = vec![0u8; count * 2];
        cur.read_exact(&mut buf)?;
        buf.pop(); buf.pop();
        // naive utf16 lossy
        let mut u16_buf = vec![0u16; count - 1];
        for i in 0..count-1 {
            u16_buf[i] = u16::from_le_bytes([buf[i*2], buf[i*2+1]]);
        }
        Ok(String::from_utf16_lossy(&u16_buf))
    }
}

fn decrypt_bytes(key: &[u8], bytes: &mut [u8]) {
    if let Ok(cipher) = aes::Aes256::new_from_slice(key) {
        for chunk in bytes.chunks_mut(16) {
            if chunk.len() == 16 {
                let mut arr = aes::cipher::generic_array::GenericArray::clone_from_slice(chunk);
                cipher.decrypt_block(&mut arr);
                chunk.copy_from_slice(&arr);
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let path = "D:\\Game\\Wuthering Waves Game\\Client\\Content\\Paks\\pakchunk3-WindowsNoEditor.pak";
    let key_hex = "0x6F80948821CA338739A24D4D9F778BCAC0996B2EF2A73897A789C68AFF05174E";
    let key_bytes = hex::decode(key_hex.trim_start_matches("0x")).unwrap();
    
    let mut file = File::open(path)?;
    let mut footer = vec![0u8; 221];
    file.seek(SeekFrom::End(-221))?;
    file.read_exact(&mut footer)?;
    let mut cur = Cursor::new(footer.as_slice());
    let mut tmp = [0u8; 26];
    cur.read_exact(&mut tmp)?; // guid, encrypted, magic, ver
    
    let index_offset = read_u64(&mut cur)?;
    let index_size = read_u64(&mut cur)?;
    
    println!("Index Offset: {}, Size: {}", index_offset, index_size);
    let align = if index_size % 16 == 0 { 0 } else { 16 - (index_size % 16) };
    let mut index_data = vec![0u8; (index_size + align) as usize];
    file.seek(SeekFrom::Start(index_offset))?;
    file.read_exact(&mut index_data)?;
    
    decrypt_bytes(&key_bytes, &mut index_data);
    
    let mut index_cur = Cursor::new(index_data.as_slice());
    let mount_point = read_string(&mut index_cur)?;
    println!("Mount Point: {}", mount_point);
    
    let num_entries = read_u32(&mut index_cur)?;
    println!("Num Entries: {}", num_entries);
    
    // PathHashIndex (V10+)
    let path_hash_offset = read_u64(&mut index_cur)?;
    let path_hash_size = read_u64(&mut index_cur)?;
    let mut path_hash = [0u8; 20];
    index_cur.read_exact(&mut path_hash)?;
    println!("PathHash Offset: {}, Size: {}", path_hash_offset, path_hash_size);
    
    // DirectoryIndex
    let dir_index_offset = read_u64(&mut index_cur)?;
    let dir_index_size = read_u64(&mut index_cur)?;
    let mut dir_hash = [0u8; 20];
    index_cur.read_exact(&mut dir_hash)?;
    println!("DirIndex Offset: {}, Size: {}", dir_index_offset, dir_index_size);
    
    // EncodedPakEntries
    let encoded_offset = read_u64(&mut index_cur)?;
    let encoded_size = read_u64(&mut index_cur)?;
    let mut encoded_hash = [0u8; 20];
    index_cur.read_exact(&mut encoded_hash)?;
    println!("EncodedEntries Offset: {}, Size: {}", encoded_offset, encoded_size);
    
    Ok(())
}
