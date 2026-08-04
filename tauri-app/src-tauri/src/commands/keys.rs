use reqwest;
use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use tauri::AppHandle;
use crate::commands::downloader::get_data_dir;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DynamicKey {
    pub guid: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WuwaKeys {
    #[serde(rename = "mainKey")]
    pub main_key: String,
    #[serde(rename = "dynamicKeys")]
    pub dynamic_keys: Vec<DynamicKey>,
}

pub fn get_all_keys(app: &AppHandle) -> Result<WuwaKeys, String> {
    let data_dir = get_data_dir(app);
    let key_cache_path = data_dir.join("keys.json");
    
    // 1. Try to load from cache first
    if key_cache_path.exists() {
        if let Ok(text) = fs::read_to_string(&key_cache_path) {
            if let Ok(keys) = serde_json::from_str::<WuwaKeys>(&text) {
                return Ok(keys);
            }
        }
    }
    
    // 2. If no cache or invalid, fetch from internet
    if let Ok(response) = reqwest::blocking::get("https://raw.githubusercontent.com/yarik0chka/wuwa-keys/main/keys.json") {
        if let Ok(text) = response.text() {
            let _ = fs::write(&key_cache_path, &text);
            if let Ok(keys) = serde_json::from_str::<WuwaKeys>(&text) {
                return Ok(keys);
            }
        }
    }
    
    Err("Failed to fetch AES keys. Please check your internet connection.".into())
}

#[tauri::command]
pub async fn fetch_latest_keys(app: AppHandle) -> Result<String, String> {
    let data_dir = get_data_dir(&app);
    let key_cache_path = data_dir.join("keys.json");
    
    let response = reqwest::blocking::get("https://raw.githubusercontent.com/yarik0chka/wuwa-keys/main/keys.json")
        .map_err(|e| format!("Network error: {}", e))?;
        
    let text = response.text().map_err(|e| format!("Failed to read response: {}", e))?;
    
    // Validate JSON before saving
    let _keys = serde_json::from_str::<WuwaKeys>(&text)
        .map_err(|e| format!("Invalid key format received: {}", e))?;
        
    fs::write(&key_cache_path, &text).map_err(|e| format!("Failed to save keys: {}", e))?;
    
    Ok("Successfully updated AES keys from GitHub.".into())
}

pub fn hex_to_bytes(hex_str: &str, expected_len: usize) -> Result<Vec<u8>, String> {
    let cleaned = hex_str.trim_start_matches("0x");
    let bytes = hex::decode(cleaned).map_err(|e| format!("Failed to parse hex: {}", e))?;
    if bytes.len() != expected_len {
        return Err(format!("Expected {} bytes, got {}", expected_len, bytes.len()));
    }
    Ok(bytes)
}

pub struct KeyManager {
    pub main_key: [u8; 32],
    pub dynamic_keys: HashMap<[u8; 16], [u8; 32]>,
}

impl KeyManager {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let keys = get_all_keys(app)?;
        
        let main_key_vec = hex_to_bytes(&keys.main_key, 32)?;
        let mut main_key = [0u8; 32];
        main_key.copy_from_slice(&main_key_vec);
        
        let mut dynamic_keys = HashMap::new();
        for dk in keys.dynamic_keys {
            if let Ok(guid_vec) = hex_to_bytes(&dk.guid, 16) {
                if let Ok(key_vec) = hex_to_bytes(&dk.key, 32) {
                    let mut guid = [0u8; 16];
                    guid.copy_from_slice(&guid_vec);
                    
                    // UE4 FGuid is 4x uint32. When represented as hex string (like in keys.json), 
                    // each uint32 is big-endian formatted, but on disk they are little-endian.
                    // We must reverse each 4-byte block to match the raw bytes from the pak footer.
                    guid[0..4].reverse();
                    guid[4..8].reverse();
                    guid[8..12].reverse();
                    guid[12..16].reverse();
                    
                    let mut key = [0u8; 32];
                    key.copy_from_slice(&key_vec);
                    dynamic_keys.insert(guid, key);
                }
            }
        }
        
        Ok(Self { main_key, dynamic_keys })
    }
    
    pub fn get_key_for_guid(&self, guid: &[u8; 16]) -> [u8; 32] {
        if guid == &[0u8; 16] {
            return self.main_key;
        }
        if let Some(key) = self.dynamic_keys.get(guid) {
            return *key;
        }
        // Fallback to main key
        self.main_key
    }
}
