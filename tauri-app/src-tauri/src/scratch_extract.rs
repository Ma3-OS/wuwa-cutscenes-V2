use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use crate::pak::PakEntry;
use byteorder::{ReadBytesExt, LE};

pub fn extract_uncompressed_file(pak_path: &str, entry: &PakEntry, out_path: &str) -> Result<(), String> {
    let mut file = File::open(pak_path).map_err(|e| e.to_string())?;
    file.seek(SeekFrom::Start(entry.offset)).map_err(|e| e.to_string())?;
    
    // Read FPakEntry
    let _offset = file.read_u64::<LE>().map_err(|e| e.to_string())?;
    let compressed = file.read_u64::<LE>().map_err(|e| e.to_string())?;
    let _uncompressed = file.read_u64::<LE>().map_err(|e| e.to_string())?;
    let compression = file.read_u32::<LE>().map_err(|e| e.to_string())?;
    
    // 20 bytes hash
    let mut hash = [0u8; 20];
    file.read_exact(&mut hash).map_err(|e| e.to_string())?;
    
    // If it has compression blocks
    if compression != 0 {
        return Err("Compressed files not supported in this simple extractor yet".into());
    }
    
    let is_encrypted = file.read_u8().map_err(|e| e.to_string())?;
    let _compression_block_size = file.read_u32::<LE>().map_err(|e| e.to_string())?;
    
    let current_pos = file.stream_position().map_err(|e| e.to_string())?;
    
    // It's uncompressed. Just copy bytes.
    let mut out_file = File::create(out_path).map_err(|e| e.to_string())?;
    
    // In Rust, we can just use std::io::copy with a take
    let mut handle = file.take(compressed);
    std::io::copy(&mut handle, &mut out_file).map_err(|e| e.to_string())?;
    
    Ok(())
}
