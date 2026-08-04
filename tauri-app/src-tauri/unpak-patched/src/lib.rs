#![allow(dead_code)]
mod entry;
mod error;
mod ext;
mod footer;
mod pak;

pub use {error::*, pak::*};

pub const MAGIC: u32 = 0x5A6F12E1;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, strum::EnumString)]
pub enum Compression {
    #[default]
    None,
    Zlib,
    Gzip,
    Oodle,
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug, strum::Display, strum::EnumIter)]
pub enum Version {
    Initial,
    NoTimestamps,
    CompressionEncryption,
    IndexEncryption,
    RelativeChunkOffsets,
    DeleteRecords,
    EncryptionKeyUuid,
    FNameBasedCompression,
    FNameBasedCompression2,
    FrozenIndex,
    PathHashIndex,
    Fnv64BugFix,
    V12,
}

impl Version {
    pub fn iter() -> VersionIter {
        <Version as strum::IntoEnumIterator>::iter()
    }

    fn as_u32(self) -> u32 {
        match self {
            Version::Initial => 1,
            Version::NoTimestamps => 2,
            Version::CompressionEncryption => 3,
            Version::IndexEncryption => 4,
            Version::RelativeChunkOffsets => 5,
            Version::DeleteRecords => 6,
            Version::EncryptionKeyUuid => 7,
            Version::FNameBasedCompression => 8,
            Version::FNameBasedCompression2 => 8,
            Version::FrozenIndex => 9,
            Version::PathHashIndex => 10,
            Version::Fnv64BugFix => 11,
            Version::V12 => 12,
        }
    }

    fn footer_size(self) -> i64 {
        let mut size = 4 + 4 + 8 + 8 + 20;
        if self >= Version::EncryptionKeyUuid { size += 16; }
        if self >= Version::IndexEncryption { size += 1; }
        if self == Version::FrozenIndex { size += 1; }
        if self >= Version::FNameBasedCompression { size += 32 * 4; }
        if self >= Version::FNameBasedCompression2 { size += 32 }
        size
    }
}

#[cfg(feature = "encryption")]
pub fn decrypt(key: Option<&aes::Aes256Dec>, bytes: &mut [u8]) -> Result<(), Error> {
    match key {
        Some(key) => {
            use aes::cipher::BlockDecrypt;
            for chunk in bytes.chunks_mut(16) {
                if chunk.len() == 16 {
                    let mut block = aes::cipher::generic_array::GenericArray::clone_from_slice(chunk);
                    key.decrypt_block(&mut block);
                    chunk.copy_from_slice(&block);
                }
            }
            Ok(())
        }
        None => Err(Error::Encrypted),
    }
}
