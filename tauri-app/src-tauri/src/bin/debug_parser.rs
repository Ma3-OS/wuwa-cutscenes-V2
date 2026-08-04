use std::fs::File;
use std::io::{Read, Cursor};
use reqwest::blocking::get;
use serde_json::Value;

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
        let mut buf = vec![0u8; len as usize];
        cur.read_exact(&mut buf)?;
        buf.pop();
        Ok(String::from_utf8_lossy(&buf).to_string())
    } else {
        let count = -len as usize;
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

fn decrypt_bytes(key: &[u8], bytes: &mut [u8]) {
    use aes::cipher::{BlockCipherDecrypt, KeyInit};
    use aes::Aes256;
    if let Ok(cipher) = Aes256::new_from_slice(key) {
        for chunk in bytes.chunks_mut(16) {
            if chunk.len() == 16 {
                let mut arr = aes::Block::clone_from_slice(chunk);
                cipher.decrypt_block(&mut arr);
                chunk.copy_from_slice(&arr);
            }
        }
    }
}

fn main() {
    // 2. Parse Pak
    let pak_path = "D:\\Game\\Wuthering Waves Game\\Client\\Saved\\Resources\\Video\\Paks\\142_2\\Video_142_2-WindowsNoEditor.pak";
    let mut file = File::open(pak_path).unwrap();
    
    use std::io::Seek;
    use std::io::SeekFrom;
    file.seek(SeekFrom::End(-221)).unwrap();
    let mut footer = [0u8; 221];
    file.read_exact(&mut footer).unwrap();
    
    let mut cur = Cursor::new(footer.as_slice());
    let mut guid = [0u8; 16]; cur.read_exact(&mut guid).unwrap();
    let mut enc = [0u8; 1]; cur.read_exact(&mut enc).unwrap();
    let mut magic = [0u8; 4]; cur.read_exact(&mut magic).unwrap();
    let mut ver = [0u8; 4]; cur.read_exact(&mut ver).unwrap();
    
    let index_offset = read_u64(&mut cur).unwrap();
    let index_size = read_u64(&mut cur).unwrap();
    let mut index_hash = [0u8; 20]; cur.read_exact(&mut index_hash).unwrap();
    
    let keys_json = get("https://raw.githubusercontent.com/yarik0chka/wuwa-keys/main/keys.json").unwrap().text().unwrap();
    let keys: Value = serde_json::from_str(&keys_json).unwrap();
    
    let game_dir = "D:\\Game\\Wuthering Waves Game\\Client\\Saved\\Resources\\Video\\Paks";
    let entries = std::fs::read_dir(game_dir).unwrap();
    let mut paks = vec![];
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            for sub_entry in std::fs::read_dir(path).unwrap() {
                let sub_entry = sub_entry.unwrap();
                if sub_entry.path().extension().map(|s| s.to_str().unwrap()) == Some("pak") {
                    paks.push(sub_entry.path());
                }
            }
        }
    }
    
    for pak_path in paks {
        println!("Checking {:?}", pak_path);
        let mut file = File::open(&pak_path).unwrap();
        let mut footer = vec![0u8; 221];
        if file.seek(SeekFrom::End(-221)).is_err() { continue; }
        if file.read_exact(&mut footer).is_err() { continue; }
        
        let mut cur = Cursor::new(footer.as_slice());
        let mut guid = [0u8; 16]; cur.read_exact(&mut guid).unwrap();
        let mut enc = [0u8; 1]; cur.read_exact(&mut enc).unwrap();
        let mut magic = [0u8; 4]; cur.read_exact(&mut magic).unwrap();
        let mut ver = [0u8; 4]; cur.read_exact(&mut ver).unwrap();
        
        let index_offset = read_u64(&mut cur).unwrap();
        let index_size = read_u64(&mut cur).unwrap();
        let mut index_hash = [0u8; 20]; cur.read_exact(&mut index_hash).unwrap();
        
        let mut rev_guid = guid.clone();
        rev_guid[0..4].reverse();
        rev_guid[4..8].reverse();
        rev_guid[8..12].reverse();
        rev_guid[12..16].reverse();
        let guid_str = hex::encode(rev_guid).to_uppercase();
        
        let mut actual_key = vec![0u8; 32];
        let mut found = false;
        if let Some(dyn_keys) = keys.get("dynamicKeys").and_then(|v| v.as_array()) {
            for k in dyn_keys {
                if let (Some(g), Some(key_val)) = (k.get("guid").and_then(|v| v.as_str()), k.get("key").and_then(|v| v.as_str())) {
                    if g.to_uppercase() == guid_str {
                        actual_key = hex::decode(key_val.trim_start_matches("0x")).unwrap();
                        found = true; break;
                    }
                }
            }
        }
        if !found {
            actual_key = hex::decode(keys.get("mainKey").unwrap().as_str().unwrap().trim_start_matches("0x")).unwrap();
        }
        
        let align = if index_size % 16 == 0 { 0 } else { 16 - (index_size % 16) };
        let mut index_data = vec![0u8; (index_size + align) as usize];
        file.seek(SeekFrom::Start(index_offset)).unwrap();
        file.read_exact(&mut index_data).unwrap();
        
        if enc[0] != 0 {
            decrypt_bytes(&actual_key, &mut index_data);
            use sha1::{Sha1, Digest};
            let mut hasher = Sha1::new();
            hasher.update(&index_data[..(index_size as usize)]);
            let result = hasher.finalize();
            println!("  Encrypted. Hash match: {}", result.as_slice() == index_hash);
            if result.as_slice() != index_hash {
                continue;
            }
        } else {
            println!("  Unencrypted.");
        }
        
        let mut idx_cur = Cursor::new(index_data.as_slice());
        let _ = read_string(&mut idx_cur).unwrap();
        let _ = read_u32(&mut idx_cur).unwrap();
        let _ = read_u64(&mut idx_cur).unwrap();
        
        let has_path_hash = read_u32(&mut idx_cur).unwrap();
        if has_path_hash != 0 {
            let _ = read_u64(&mut idx_cur).unwrap();
            let _ = read_u64(&mut idx_cur).unwrap();
            let mut tmp = [0u8; 20]; idx_cur.read_exact(&mut tmp).unwrap();
        }
        
        let has_dir = read_u32(&mut idx_cur).unwrap();
        let mut dir_offset = 0;
        let mut dir_size = 0;
        if has_dir != 0 {
            dir_offset = read_u64(&mut idx_cur).unwrap();
            dir_size = read_u64(&mut idx_cur).unwrap();
            let mut tmp = [0u8; 20]; idx_cur.read_exact(&mut tmp).unwrap();
        }
        
        let enc_size = read_u32(&mut idx_cur).unwrap();
        let mut encoded_data = vec![0u8; enc_size as usize];
        idx_cur.read_exact(&mut encoded_data).unwrap();
        
        let mut dir_data = vec![];
        if has_dir != 0 {
            let align_dir = if dir_size % 16 == 0 { 0 } else { 16 - (dir_size % 16) };
            dir_data = vec![0u8; (dir_size + align_dir) as usize];
            file.seek(SeekFrom::Start(dir_offset)).unwrap();
            file.read_exact(&mut dir_data).unwrap();
            if enc[0] != 0 {
                decrypt_bytes(&actual_key, &mut dir_data);
            }
        }
        
        let mut dir_cur = Cursor::new(dir_data.as_slice());
        let num_dirs = read_u32(&mut dir_cur).unwrap_or(0);
        for _ in 0..num_dirs {
            let _dir_name = read_string(&mut dir_cur).unwrap();
            let num_files = read_u32(&mut dir_cur).unwrap_or(0);
            for _ in 0..num_files {
                let file_name = read_string(&mut dir_cur).unwrap_or_default();
                let file_enc_offset = read_u32(&mut dir_cur).unwrap_or(0);
                if file_name.contains("M3_10_04") {
                    println!("  FOUND {} at {}", file_name, file_enc_offset);
                    let mut ecur = Cursor::new(encoded_data.as_slice());
                    ecur.set_position(file_enc_offset as u64);
                    let mut bitfield = read_u32(&mut ecur).unwrap_or(0);
                    let orig_bitfield = bitfield;
                    bitfield = (bitfield >> 16) & 0x3F | (bitfield & 0xFFFF) << 6 | (bitfield & (1 << 28)) >> 6 |
                               (bitfield & 0x0FC00000) << 1 | (bitfield & 0xC0000000) >> 1 | (bitfield & 0x20000000) << 2;
                    let mut custom_data = [0u8; 1];
                    ecur.read_exact(&mut custom_data).unwrap();
                    
                    let bIsOffset32BitSafe = (bitfield & (1 << 31)) != 0;
                    let bIsUncompressedSize32BitSafe = (bitfield & (1 << 30)) != 0;
                    let bIsSize32BitSafe = (bitfield & (1 << 29)) != 0;
                    
                    println!("    bitfield={:08X} (orig={:08X}), offset32={}, uncomp32={}, size32={}", bitfield, orig_bitfield, bIsOffset32BitSafe, bIsUncompressedSize32BitSafe, bIsSize32BitSafe);
                    
                    let offset_read = if bIsOffset32BitSafe { read_u32(&mut ecur).unwrap_or(0) as u64 } else { read_u64(&mut ecur).unwrap_or(0) };
                    let uncomp_size_read = if bIsUncompressedSize32BitSafe { read_u32(&mut ecur).unwrap_or(0) as u64 } else { read_u64(&mut ecur).unwrap_or(0) };
                    
                    let uncomp_size = offset_read;
                    let off = uncomp_size_read;
                    
                    let comp_method = (bitfield >> 23) & 0x3F;
                    let size = if comp_method != 0 {
                        if bIsSize32BitSafe { read_u32(&mut ecur).unwrap_or(0) as u64 } else { read_u64(&mut ecur).unwrap_or(0) }
                    } else {
                        uncomp_size
                    };
                    let is_enc = (bitfield & (1 << 22)) != 0; // Not sure if bit 22 is enc, but let's see. Wait, in UE4 bit 22 is usually isEncrypted.
                    let comp = comp_method;
                    println!("    offset={}, size={}, uncomp_size={}, comp={}, is_enc={}", off, size, uncomp_size, comp, is_enc);
                    
                    // EXTRACT LOGIC
                    let mut payload = vec![0u8; size as usize];
                    file.seek(SeekFrom::Start(off)).unwrap();
                    
                    let mut offset_check = [0u8; 8];
                    file.read_exact(&mut offset_check).unwrap();
                    let read_off = u64::from_le_bytes(offset_check);
                    
                    let mut data_offset = off;
                    if read_off == off {
                        let mut rest = [0u8; 16]; file.read_exact(&mut rest).unwrap();
                        let mut cb = [0u8; 4]; file.read_exact(&mut cb).unwrap();
                        let cmethod = u32::from_le_bytes(cb);
                        let mut hash = [0u8; 20]; file.read_exact(&mut hash).unwrap();
                        if cmethod != 0 {
                            let mut cnt = [0u8; 4]; file.read_exact(&mut cnt).unwrap();
                            let count = u32::from_le_bytes(cnt);
                            file.seek(SeekFrom::Current((count * 16) as i64)).unwrap();
                        }
                        let mut encb = [0u8; 5]; file.read_exact(&mut encb).unwrap();
                        data_offset = file.stream_position().unwrap();
                    } else {
                        file.seek(SeekFrom::Start(off)).unwrap();
                        data_offset = off;
                    }
                    println!("    data_offset={}", data_offset);
                    
                    let bytes_to_read = size as usize;
                    let file_len = file.metadata().unwrap().len();
                    if data_offset + (bytes_to_read as u64) > file_len {
                        println!("    WARNING: data_offset {} + size {} > file_len {}", data_offset, bytes_to_read, file_len);
                        let rem = file_len.saturating_sub(data_offset) as usize;
                        let read_len = std::cmp::min(bytes_to_read, rem);
                        println!("    Reading {} bytes...", read_len);
                        let mut buf = vec![0u8; read_len];
                        if let Err(e) = file.read_exact(&mut buf) {
                            println!("    READ EXACT ERROR: {:?}", e);
                        } else {
                            println!("    Read success!");
                        }
                    } else {
                        let mut buf = vec![0u8; bytes_to_read];
                        if let Err(e) = file.read_exact(&mut buf) {
                            println!("    READ EXACT ERROR (NORMAL): {:?}", e);
                        } else {
                            println!("    Read success!");
                        }
                    }
                }
            }
        }
    }
}
