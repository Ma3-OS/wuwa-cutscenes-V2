pub mod parser;

pub const MAGIC: u32 = 0x5A6F12E1;


#[derive(Debug, Clone)]
pub struct PakEntry {
    pub offset: u64,
    pub size: u64,
    pub uncompressed_size: u64,
    pub compression_method: u32,
    pub is_encrypted: bool,
    pub custom_data: u8,
    // (Size, CompressedStart)
    pub compression_blocks: Vec<(u64, u64)>,
}

#[derive(Debug)]
pub struct PakInfo {
    pub magic: u32,
    pub version: u32,
    pub index_offset: u64,
    pub index_size: u64,
    pub index_hash: [u8; 20],
    pub encrypted_index: bool,
    pub encryption_key_guid: [u8; 16],
    pub compression_methods: Vec<String>,
}

#[derive(Debug)]
pub struct PakFile {
    pub path: String,
    pub info: PakInfo,
    pub entries: std::collections::HashMap<String, PakEntry>,
    pub mount_point: String,
    pub key: Option<Vec<u8>>,
}
