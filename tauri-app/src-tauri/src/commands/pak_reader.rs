use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::AppHandle;

use crate::pak::parser::{parse_pak, PakError};
use crate::pak::PakFile;
use crate::commands::log::emit_log;
use crate::commands::keys::KeyManager;

pub struct PakManager {
    app: AppHandle,
    paks_dirs: Vec<PathBuf>,
    pub paks: Mutex<Vec<PakFile>>,
}

fn get_all_paks_in_dir(dir: &Path) -> Vec<PathBuf> {
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

impl PakManager {
    pub fn new(app: &AppHandle, paks_dirs: Vec<PathBuf>) -> Result<Self, String> {
        Ok(Self {
            app: app.clone(),
            paks_dirs,
            paks: Mutex::new(Vec::new()),
        })
    }

    pub fn mount_all(&self, key_manager: &KeyManager) -> Result<(), String> {
        let mut paks_vec = self.paks.lock().unwrap();
        paks_vec.clear();

        let mut loaded = 0;
        let mut failed = 0;

        for dir in &self.paks_dirs {
            if !dir.exists() {
                emit_log(&self.app, &format!("Warning: Pak directory does not exist: {:?}", dir));
                continue;
            }
            
            let all_paks = get_all_paks_in_dir(dir);
            emit_log(&self.app, &format!("Found {} pak files in {:?}", all_paks.len(), dir));
            
            for path in all_paks {
                let path_str = path.to_string_lossy().to_string();
                let file_name = path.file_name().unwrap().to_string_lossy();
                
                match parse_pak(&path_str, key_manager) {
                    Ok(pak) => {
                        emit_log(&self.app, &format!("Mounted {} ({} entries)", file_name, pak.entries.len()));
                        paks_vec.push(pak);
                        loaded += 1;
                    }
                    Err(e) => {
                        emit_log(&self.app, &format!("Failed to mount {}: {:?}", file_name, e));
                        failed += 1;
                    }
                }
            }
        }
        
        emit_log(&self.app, &format!("Finished: {} mounted, {} failed.", loaded, failed));
        Ok(())
    }
    
    pub fn get_video_files(&self) -> Vec<String> {
        let mut video_files = Vec::new();
        let paks = self.paks.lock().unwrap();
        for pak in paks.iter() {
            for (path, _) in &pak.entries {
                if path.ends_with(".mp4") || path.ends_with(".bk2") || path.ends_with(".mkv") || path.ends_with(".avi") {
                    video_files.push(path.clone());
                }
            }
        }
        video_files.sort();
        video_files.dedup();
        video_files
    }

    pub fn extract_all_audio_banks(&self, out_dir: &Path) -> Result<usize, String> {
        let paks = self.paks.lock().unwrap();
        let mut extracted_count = 0;
        
        for pak in paks.iter() {
            for (path, entry) in &pak.entries {
                if path.ends_with(".bnk") || path.ends_with(".wem") || path.ends_with(".txtp") {
                    // Extract relative path after WwiseAudio_Generated/
                    if let Some(idx) = path.find("WwiseAudio_Generated/") {
                        let rel_path = &path[idx + "WwiseAudio_Generated/".len()..];
                        let dest_path = out_dir.join(rel_path);
                        
                        if dest_path.exists() {
                            continue; // Already extracted
                        }
                        
                        if let Some(parent) = dest_path.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        
                        match crate::pak::parser::extract_file(&pak.path, entry, &pak.key, dest_path.to_str().unwrap()) {
                            Ok(_) => extracted_count += 1,
                            Err(e) => emit_log(&self.app, &format!("Failed to extract audio bank {}: {:?}", path, e)),
                        }
                    }
                }
            }
        }
        Ok(extracted_count)
    }
}

pub static PAK_MANAGER: OnceLock<Mutex<Option<PakManager>>> = OnceLock::new();

#[tauri::command]
pub fn test_pak_mount(app: AppHandle, state: tauri::State<'_, crate::config::ConfigState>) -> Result<String, String> {
    emit_log(&app, "Starting V3 Pak Mounting...");
    
    let config = state.0.lock().unwrap().clone();
    let game_dir = config.game_dir;
    if game_dir.is_empty() {
        return Err("Game directory not set".into());
    }
    
    let paks_dir_1 = PathBuf::from(&game_dir).join("Client/Content/Paks");
    let paks_dir_2 = PathBuf::from(&game_dir).join("Client/Saved/Resources/Video/Paks");
    
    let dirs = vec![paks_dir_1, paks_dir_2];
    let manager = PakManager::new(&app, dirs)?;
    
    emit_log(&app, "Fetching AES keys from GitHub...");
    let key_manager = KeyManager::new(&app)?;
    emit_log(&app, &format!("Loaded main key and {} dynamic keys.", key_manager.dynamic_keys.len()));
    
    manager.mount_all(&key_manager)?;
    
    let global_manager = PAK_MANAGER.get_or_init(|| Mutex::new(None));
    *global_manager.lock().unwrap() = Some(manager);
    
    Ok("Test completed!".to_string())
}

#[tauri::command]
pub fn get_pak_files(app: AppHandle, state: tauri::State<crate::config::ConfigState>) -> Result<Vec<String>, String> {
    let global_manager = PAK_MANAGER.get_or_init(|| Mutex::new(None));
    let mut manager_lock = global_manager.lock().unwrap();
    
    if manager_lock.is_none() {
        let config = state.0.lock().unwrap().clone();
        let game_dir = config.game_dir;
        if game_dir.is_empty() {
            return Ok(vec![]);
        }
        
        let paks_dir_1 = PathBuf::from(&game_dir).join("Client/Content/Paks");
        let paks_dir_2 = PathBuf::from(&game_dir).join("Client/Saved/Resources/Video/Paks");
        let dirs = vec![paks_dir_1, paks_dir_2];
        
        let manager = PakManager::new(&app, dirs).map_err(|e| e.to_string())?;
        if let Ok(key_manager) = KeyManager::new(&app) {
            let _ = manager.mount_all(&key_manager);
        }
        *manager_lock = Some(manager);
    }
    
    if let Some(manager) = manager_lock.as_ref() {
        Ok(manager.get_video_files())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn process_cutscene(app: AppHandle, state: tauri::State<'_, crate::config::ConfigState>, video_name: String) -> Result<String, String> {
    let global_manager = PAK_MANAGER.get().ok_or("Pak manager not initialized")?;
    let manager = global_manager.lock().unwrap();
    if manager.is_none() {
        return Err("Paks not mounted".into());
    }
    let manager = manager.as_ref().unwrap();
    
    let paks = manager.paks.lock().unwrap();
    for pak in paks.iter() {
        if let Some(entry) = pak.entries.get(&video_name) {
            let data_dir = crate::commands::downloader::get_data_dir(&app);
            let out_dir = data_dir.join("output");
            let _ = std::fs::create_dir_all(&out_dir);
            
            let file_name = std::path::Path::new(&video_name).file_name().unwrap().to_string_lossy();
            let out_path = out_dir.join(format!("{}", file_name));
            
            let mb_size = entry.uncompressed_size as f64 / 1024.0 / 1024.0;
            
            if out_path.exists() {
                emit_log(&app, &format!("Video already extracted ({:.2} MB). Skipping extraction.", mb_size));
                return Ok(out_path.to_string_lossy().to_string());
            }
            
            emit_log(&app, &format!("Extracting cutscene... ({:.2} MB)", mb_size));
            match crate::pak::parser::extract_file(&pak.path, entry, &pak.key, out_path.to_str().unwrap()) {
                Ok(path) => {
                    emit_log(&app, "Video extraction successful.");
                    return Ok(path);
                }
                Err(e) => return Err(format!("Extraction failed: {:?}", e)),
            }
        }
    }
    
    Err("Video not found in any mounted Pak".into())
}

#[tauri::command]
pub async fn extract_audio_banks(app: AppHandle) -> Result<String, String> {
    emit_log(&app, "Scanning for Wwise audio banks in mounted paks...");
    
    let global_manager = PAK_MANAGER.get().ok_or("Pak manager not initialized. Please mount paks first.")?;
    let manager = global_manager.lock().unwrap();
    
    if manager.is_none() {
        return Err("Pak manager not initialized. Please mount paks first.".into());
    }
    
    let data_dir = crate::commands::downloader::get_data_dir(&app);
    let out_dir = data_dir.join("wwise_audio");
    
    if !out_dir.exists() {
        let _ = fs::create_dir_all(&out_dir);
    }
    
    let extracted_count = manager.as_ref().unwrap().extract_all_audio_banks(&out_dir)?;
    
    if extracted_count > 0 {
        emit_log(&app, &format!("Successfully extracted {} audio bank files.", extracted_count));
    } else {
        emit_log(&app, "Audio banks are already extracted or none found.");
    }
    
    Ok(out_dir.to_string_lossy().to_string())
}
