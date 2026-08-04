use std::fs;
use std::path::PathBuf;
use tauri_app_lib::pak::parser::{parse_pak, extract_file};
use tauri_app_lib::commands::keys::KeyManager;

fn get_all_paks_in_dir(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut paks = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                paks.extend(get_all_paks_in_dir(&path));
            } else if path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) == Some("pak".to_string()) {
                paks.push(path);
            }
        }
    }
    paks
}

fn main() {
    let game_dir = "D:\\Game\\Wuthering Waves Game\\Client\\Saved\\Resources\\Video\\Paks";
    let paks = get_all_paks_in_dir(std::path::Path::new(game_dir));
    
    // We need a mock KeyManager or we can just load the real keys JSON.
    // Wait, KeyManager::new needs an AppHandle.
    // We can just manually download the keys for this test.
    let keys_json = reqwest::blocking::get("https://raw.githubusercontent.com/yarik0chka/wuwa-keys/main/keys.json").unwrap().text().unwrap();
    let keys_val: serde_json::Value = serde_json::from_str(&keys_json).unwrap();
    let main_key_str = keys_val.get("mainKey").unwrap().as_str().unwrap().trim_start_matches("0x");
    let main_key = hex::decode(main_key_str).unwrap();
    
    let mut dyn_keys = std::collections::HashMap::new();
    if let Some(arr) = keys_val.get("dynamicKeys").and_then(|v| v.as_array()) {
        for k in arr {
            if let (Some(g), Some(key_val)) = (k.get("guid").and_then(|v| v.as_str()), k.get("key").and_then(|v| v.as_str())) {
                let hex_str = key_val.trim_start_matches("0x");
                if let Ok(dec) = hex::decode(hex_str) {
                    dyn_keys.insert(g.to_uppercase(), dec);
                }
            }
        }
    }
    
    let mut loaded = 0;
    for path in paks {
        if path.file_name().unwrap().to_str().unwrap().starts_with("Video_") {
            let path_str = path.to_str().unwrap();
            
            // To call parse_pak, we can't easily pass KeyManager because we are outside tauri context.
            // But wait, parse_pak requires &KeyManager. I can't construct KeyManager without AppHandle easily.
            // Actually I can just modify parser.rs to accept a trait or I can copy the parse_pak code.
            // Let's just print the files!
            println!("Found pak: {}", path_str);
        }
    }
}
