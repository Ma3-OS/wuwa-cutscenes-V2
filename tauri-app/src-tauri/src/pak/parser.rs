use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor};
use aes::cipher::{BlockCipherDecrypt, KeyInit};
use sha1::{Sha1, Digest};
use super::{PakFile, PakInfo, PakEntry};
use crate::commands::keys::KeyManager;

#[derive(Debug)]
pub enum PakError {
    Io(std::io::Error),
    InvalidMagic,
    UnsupportedVersion,
    DecryptionFailed,
    ParseError(String),
}

impl From<std::io::Error> for PakError {
    fn from(err: std::io::Error) -> Self {
        PakError::Io(err)
    }
}

pub fn decrypt_bytes(key: &[u8], bytes: &mut [u8]) -> Result<(), PakError> {
    if let Ok(cipher) = aes::Aes256::new_from_slice(key) {
        for chunk in bytes.chunks_mut(16) {
            if chunk.len() == 16 {
                let mut arr = aes::Block::clone_from_slice(chunk);
                cipher.decrypt_block(&mut arr);
                chunk.copy_from_slice(&arr);
            }
        }
        Ok(())
    } else {
        Err(PakError::DecryptionFailed)
    }
}

pub fn read_u32(cur: &mut Cursor<&[u8]>) -> std::io::Result<u32> {
    let mut buf = [0u8; 4];
    cur.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_u64(cur: &mut Cursor<&[u8]>) -> std::io::Result<u64> {
    let mut buf = [0u8; 8];
    cur.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub fn read_string(cur: &mut Cursor<&[u8]>) -> std::io::Result<String> {
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
        let mut u16_buf = vec![0u16; count - 1];
        for i in 0..count-1 {
            u16_buf[i] = u16::from_le_bytes([buf[i*2], buf[i*2+1]]);
        }
        Ok(String::from_utf16_lossy(&u16_buf))
    }
}

pub fn parse_pak(path: &str, key_manager: &KeyManager) -> Result<PakFile, PakError> {
    let mut file = File::open(path)?;
    let mut footer = vec![0u8; 221];
    file.seek(SeekFrom::End(-221))?;
    file.read_exact(&mut footer)?;
    let mut cur = Cursor::new(footer.as_slice());
    
    let mut guid = [0u8; 16]; cur.read_exact(&mut guid)?;
    let mut enc = [0u8; 1]; cur.read_exact(&mut enc)?;
    let mut magic = [0u8; 4]; cur.read_exact(&mut magic)?;
    if u32::from_le_bytes(magic) != super::MAGIC { return Err(PakError::InvalidMagic); }
    
    let mut ver = [0u8; 4]; cur.read_exact(&mut ver)?;
    let version = u32::from_le_bytes(ver);
    
    let index_offset = read_u64(&mut cur)?;
    let index_size = read_u64(&mut cur)?;
    let mut index_hash = [0u8; 20]; cur.read_exact(&mut index_hash)?;
    
    let info = PakInfo {
        magic: super::MAGIC,
        version,
        index_offset,
        index_size,
        index_hash,
        encrypted_index: enc[0] != 0,
        encryption_key_guid: guid,
        compression_methods: vec![],
    };
    
    let align = if index_size % 16 == 0 { 0 } else { 16 - (index_size % 16) };
    let mut index_data = vec![0u8; (index_size + align) as usize];
    file.seek(SeekFrom::Start(index_offset))?;
    file.read_exact(&mut index_data)?;
    
    let actual_key = key_manager.get_key_for_guid(&guid);
    let mut key_used = None;

    if info.encrypted_index {
        decrypt_bytes(&actual_key, &mut index_data)?;
        
        let mut hasher = Sha1::new();
        hasher.update(&index_data[..(index_size as usize)]);
        let result = hasher.finalize();
        if result.as_slice() != index_hash {
            return Err(PakError::DecryptionFailed);
        }
        key_used = Some(actual_key.to_vec());
    }
    
    let mut idx_cur = Cursor::new(index_data.as_slice());
    let mount_point = match read_string(&mut idx_cur) {
        Ok(s) => s,
        Err(_) => return Err(PakError::ParseError("Mount point read error".into())),
    };
    
    let _num_entries = read_u32(&mut idx_cur)?;
    let _path_hash_seed = read_u64(&mut idx_cur)?;
    
    let has_path_hash_index = read_u32(&mut idx_cur)?;
    if has_path_hash_index != 0 {
        let _path_hash_offset = read_u64(&mut idx_cur)?;
        let _path_hash_size = read_u64(&mut idx_cur)?;
        let mut _path_hash = [0u8; 20]; idx_cur.read_exact(&mut _path_hash)?;
    }
    
    let has_dir_index = read_u32(&mut idx_cur)?;
    let mut dir_index_offset = 0;
    let mut dir_index_size = 0;
    if has_dir_index != 0 {
        dir_index_offset = read_u64(&mut idx_cur)?;
        dir_index_size = read_u64(&mut idx_cur)?;
        let mut _dir_hash = [0u8; 20]; idx_cur.read_exact(&mut _dir_hash)?;
    }
    
    let encoded_size = read_u32(&mut idx_cur)?;
    if encoded_size > 500_000_000 { return Err(PakError::ParseError("encoded_size too large".into())); }
    
    let mut encoded_data = vec![0u8; encoded_size as usize];
    idx_cur.read_exact(&mut encoded_data)?;
    if info.encrypted_index {
        let align_enc = if encoded_size % 16 == 0 { 0 } else { 16 - (encoded_size % 16) };
        if align_enc > 0 {
            // Need to read extra alignment bytes? 
            // Wait, encoded_size is the exact size. UE4 aligns the padding on the file.
            // Let's just pass it to decrypt_bytes, but decrypt_bytes requires length % 16 == 0!
            // Wait, unpak says: `let mut data = index.read_len(size as usize)?;` and then decrypts it!
            // BUT index is aligned. So encoded_size might not be aligned, but the buffer needs to be aligned.
        }
    }
    
    let mut dir_data = vec![];
    if has_dir_index != 0 {
        if dir_index_size > 1_000_000_000 { return Err(PakError::ParseError("dir_index_size too large".into())); }
        let align_dir = if dir_index_size % 16 == 0 { 0 } else { 16 - (dir_index_size % 16) };
        dir_data = vec![0u8; (dir_index_size + align_dir) as usize];
        file.seek(SeekFrom::Start(dir_index_offset))?;
        file.read_exact(&mut dir_data)?;
        if info.encrypted_index {
            decrypt_bytes(&actual_key, &mut dir_data)?;
        }
    }
    
    let mut entries = std::collections::HashMap::new();
    if !dir_data.is_empty() {
        let mut dir_cur = Cursor::new(dir_data.as_slice());
        
        let num_directories = read_u32(&mut dir_cur).unwrap_or(0);
        
        for _ in 0..num_directories {
            let dir_name = read_string(&mut dir_cur).unwrap_or_default();
            let num_files = read_u32(&mut dir_cur).unwrap_or(0);
            for _ in 0..num_files {
                let file_name = read_string(&mut dir_cur).unwrap_or_default();
                let file_encoded_offset = read_u32(&mut dir_cur).unwrap_or(0);
                
                let mut enc_cur = Cursor::new(encoded_data.as_slice());
                enc_cur.set_position(file_encoded_offset as u64);
                
                let mut bitfield = read_u32(&mut enc_cur).unwrap_or(0);
                
                // Wuthering Waves specific deobfuscation
                bitfield = (bitfield >> 16) & 0x3F | (bitfield & 0xFFFF) << 6 | (bitfield & (1 << 28)) >> 6 |
                           (bitfield & 0x0FC00000) << 1 | (bitfield & 0xC0000000) >> 1 | (bitfield & 0x20000000) << 2;
                
                let mut custom_data = [0u8; 1];
                let _ = enc_cur.read_exact(&mut custom_data);
                
                let b_is_offset32_bit_safe = (bitfield & (1 << 31)) != 0;
                let b_is_uncompressed_size32_bit_safe = (bitfield & (1 << 30)) != 0;
                let b_is_size32_bit_safe = (bitfield & (1 << 29)) != 0;
                
                let offset_read = if b_is_offset32_bit_safe { read_u32(&mut enc_cur).unwrap_or(0) as u64 } else { read_u64(&mut enc_cur).unwrap_or(0) };
                let uncomp_size_read = if b_is_uncompressed_size32_bit_safe { read_u32(&mut enc_cur).unwrap_or(0) as u64 } else { read_u64(&mut enc_cur).unwrap_or(0) };
                
                // In WuWa, Offset and UncompressedSize are swapped!
                let offset = uncomp_size_read;
                let uncompressed_size = offset_read;
                
                let comp_method = (bitfield >> 23) & 0x3F;
                let size = if comp_method != 0 {
                    if b_is_size32_bit_safe { read_u32(&mut enc_cur).unwrap_or(0) as u64 } else { read_u64(&mut enc_cur).unwrap_or(0) }
                } else {
                    uncompressed_size
                };
                
                let is_encrypted = (bitfield & (1 << 22)) != 0;
                let compression_method = comp_method;
                
                let full_path = if dir_name.is_empty() {
                    file_name.clone()
                } else {
                    format!("{}{}", dir_name, file_name)
                };
                
                entries.insert(full_path, PakEntry {
                    offset,
                    size,
                    uncompressed_size,
                    compression_method,
                    is_encrypted,
                    custom_data: 0,
                    compression_blocks: vec![],
                });
            }
        }
    }
    
    Ok(PakFile {
        path: path.to_string(),
        info,
        entries,
        mount_point,
        key: key_used,
    })
}

pub fn extract_file(
    pak_path: &str,
    entry: &PakEntry,
    key_used: &Option<Vec<u8>>,
    out_path: &str,
) -> Result<String, PakError> {
    use std::io::Write;
    let mut file = File::open(pak_path)?;
    let file_len = file.metadata()?.len();
    
    if entry.offset >= file_len {
        return Err(PakError::ParseError(format!("Offset {} exceeds file length {}", entry.offset, file_len)));
    }
    
    file.seek(SeekFrom::Start(entry.offset))?;
    // Check if the offset points to FPakEntry or directly to raw data
    // In standard UE4, the first 8 bytes of FPakEntry is the offset itself.
    let mut offset_check = [0u8; 8];
    file.read_exact(&mut offset_check)?;
    let read_off = u64::from_le_bytes(offset_check);
    
    let mut data_offset = entry.offset;
    
    if read_off == entry.offset {
        // It's a standard FPakEntry, skip the payload header
        // Since we already read 8 bytes (offset), we read the next 8 bytes (size)
        let mut size_buf = [0u8; 8]; file.read_exact(&mut size_buf)?; // UncompressedSize (8)
        let mut cb = [0u8; 4]; file.read_exact(&mut cb)?; // CompressionMethod (4) - wait, this assumes 4 bytes which might be wrong for v11, but for standard paks it might be 4.
        let cmethod = u32::from_le_bytes(cb);
        let mut hash = [0u8; 20]; file.read_exact(&mut hash)?; // Hash (20)
        
        if cmethod != 0 {
            let mut cnt = [0u8; 4]; file.read_exact(&mut cnt)?;
            let count = u32::from_le_bytes(cnt);
            file.seek(SeekFrom::Current((count * 16) as i64))?;
        }
        let mut encb = [0u8; 5]; file.read_exact(&mut encb)?; // bEncrypted (1) + CompressionBlockSize (4)
        data_offset = file.stream_position()?;
    } else {
        // It already points to the raw data (Wuthering Waves custom format)
        data_offset = entry.offset;
    }
    
    file.seek(SeekFrom::Start(data_offset))?;
    
    // Safety check for EOF
    let mut bytes_to_read = entry.size as usize;
    if entry.is_encrypted {
        bytes_to_read = (bytes_to_read + 15) & !15;
    }
    
    let remaining = file_len.saturating_sub(data_offset) as usize;
    let read_len = std::cmp::min(bytes_to_read, remaining);
    
    let mut data = vec![0u8; read_len];
    if let Err(e) = file.read_exact(&mut data) {
        return Err(PakError::ParseError(format!("Failed reading payload ({} bytes at offset {}): {}", read_len, data_offset, e)));
    }
    
    if entry.is_encrypted {
        if let Some(key) = key_used {
            decrypt_bytes(key, &mut data)?;
            data.truncate(entry.uncompressed_size as usize);
        } else {
            return Err(PakError::DecryptionFailed);
        }
    }
    
    let mut out_file = File::create(out_path)?;
    out_file.write_all(&data)?;
    
    Ok(out_path.to_string())
}
