use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub ffmpeg_path: String,
    pub vgmstream_path: String,
    pub game_dir: String,
    pub char_selection: String,
    pub locale_selection: String,
    pub subtitle_lang: String,
    pub subtitle_mode: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ffmpeg_path: String::new(),
            vgmstream_path: String::new(),
            game_dir: String::new(),
            char_selection: "Girl".to_string(),
            locale_selection: "ja".to_string(),
            subtitle_lang: "en".to_string(),
            subtitle_mode: "Soft-sub".to_string(),
        }
    }
}

pub struct ConfigState(pub Mutex<AppConfig>);

fn get_config_path(app: &AppHandle) -> PathBuf {
    app.path().app_config_dir().unwrap().join("config.json")
}

pub fn init_config(app: &AppHandle) {
    let config_path = get_config_path(app);
    
    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let config = if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppConfig::default()
        }
    } else {
        AppConfig::default()
    };

    app.manage(ConfigState(Mutex::new(config)));
}

#[tauri::command]
pub fn get_config(state: tauri::State<ConfigState>) -> AppConfig {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn update_config(app: tauri::AppHandle, state: tauri::State<ConfigState>, new_config: AppConfig) -> Result<(), String> {
    let mut config = state.0.lock().unwrap();
    *config = new_config.clone();
    
    let config_path = get_config_path(&app);
    let content = serde_json::to_string_pretty(&new_config).map_err(|e| e.to_string())?;
    fs::write(config_path, content).map_err(|e| e.to_string())?;
    
    Ok(())
}
