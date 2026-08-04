use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use crate::commands::downloader::get_tools_dir;

#[derive(serde::Serialize)]
pub struct DependencyStatus {
    ffmpeg: bool,
    wwiser: bool,
    vgmstream: bool,
    json_data: bool,
}

#[tauri::command]
pub fn check_dependencies(app: AppHandle) -> DependencyStatus {
    let tools_dir = get_tools_dir(&app);
    let data_dir = crate::commands::downloader::get_data_dir(&app);
    
    let ffmpeg_exists = tools_dir.join("ffmpeg.exe").exists();
    let wwiser_exists = tools_dir.join("wwiser-master").join("wwiser.py").exists();
    let vgmstream_exists = tools_dir.join("vgmstream-cli.exe").exists();
    
    let json_exists = data_dir.join("videodata.json").exists() &&
                      data_dir.join("videosound.json").exists() &&
                      data_dir.join("MultiText.json").exists();
                      
    DependencyStatus {
        ffmpeg: ffmpeg_exists,
        wwiser: wwiser_exists,
        vgmstream: vgmstream_exists,
        json_data: json_exists,
    }
}

                // Removed Auto-Detect as per user request (FModel output is separate)

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    let p = PathBuf::from(path);
    if p.exists() {
        open::that(p).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Folder does not exist".to_string())
    }
}

#[tauri::command]
pub fn open_output_dir(app: AppHandle) -> Result<(), String> {
    let output_dir = crate::commands::downloader::get_data_dir(&app).join("output");
    let _ = fs::create_dir_all(&output_dir);
    open::that(output_dir).map_err(|e| e.to_string())?;
    Ok(())
}
