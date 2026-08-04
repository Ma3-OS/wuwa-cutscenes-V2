use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

use crate::config::ConfigState;
use crate::commands::downloader::get_data_dir;
use crate::commands::log::emit_log;

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoInfo {
    pub cg_name: String,
    pub cg_file: String,
    pub girl_or_boy: String,
    pub sounds: Vec<String>,
    pub srt_path: Option<String>,
}

#[tauri::command]
pub async fn generate_single_video_info(
    app: AppHandle,
    state: tauri::State<'_, ConfigState>,
    mp4_path: String,
    locale: String,
    girl_or_boy: String,
) -> Result<VideoInfo, String> {
    let config = state.0.lock().unwrap().clone();
    let data_dir = get_data_dir(&app);
    let wwise_path = data_dir.join("wwise_audio");
    let mp4 = PathBuf::from(&mp4_path);

    if !mp4.exists() {
        return Err(format!("Video file not found: {}", mp4_path));
    }

    emit_log(&app, &format!("Parsing metadata for {}...", mp4.file_name().unwrap().to_string_lossy()));

    let videodata_path = data_dir.join("videodata.json");
    let videosound_path = data_dir.join("videosound.json");

    if !videodata_path.exists() || !videosound_path.exists() {
        return Err("JSON metadata not found. Please download tools first.".into());
    }

    let videodata_str = fs::read_to_string(&videodata_path).map_err(|e| e.to_string())?;
    let videosound_str = fs::read_to_string(&videosound_path).map_err(|e| e.to_string())?;

    let videodata: Vec<Value> = serde_json::from_str(&videodata_str).map_err(|e| e.to_string())?;
    let videosound: Vec<Value> = serde_json::from_str(&videosound_str).map_err(|e| e.to_string())?;

    let base_name = mp4.file_stem().unwrap().to_string_lossy().to_string().to_lowercase();
    
    // Find matching CgName
    let mut target_cg_name = String::new();
    let target_gender_num = if girl_or_boy == "Girl" { 0 } else { 1 };
    
    for item in videodata {
        if let Some(cg_file) = item.get("CgFile").and_then(|v| v.as_str()) {
            let fn_name = cg_file.split('/').last().unwrap().split('.').next().unwrap().to_lowercase();
            // Basic matching - the original python had a fixup map, but we'll try loose matching first
            if fn_name == base_name || base_name.contains(&fn_name) || fn_name.contains(&base_name) {
                target_cg_name = item.get("CgName").and_then(|v| v.as_str()).unwrap_or("").to_string();
                break;
            }
        }
    }

    if target_cg_name.is_empty() {
        return Err(format!("Could not find metadata for video: {}", base_name));
    }

    emit_log(&app, &format!("Found CgName: {}", target_cg_name));

    // Find sounds
    let mut events = Vec::new();
    for item in videosound {
        if let Some(cg_name) = item.get("CgName").and_then(|v| v.as_str()) {
            if cg_name == target_cg_name {
                if let Some(event_path) = item.get("EventPath").and_then(|v| v.as_str()) {
                    let event = event_path.split('/').last().unwrap().split('.').next().unwrap().to_string();
                    events.push(event);
                }
            }
        }
    }
    
    // Scan wwise dir for matching txtp
    let mut sounds = Vec::new();
    let txtp_cache = index_directory(&wwise_path, "txtp");
    
    // Simplistic sound matching
    let gender_param = if girl_or_boy == "Girl" { "(3313202977=2204441813)" } else { "(3313202977=3111576190)" };
    
    for event in events {
        let mut matched = Vec::new();
        for txtp in &txtp_cache {
            if txtp.to_lowercase().contains(&event.to_lowercase()) {
                matched.push(txtp.clone());
            }
        }
        
        for m in matched {
            // Apply locale and gender filters simply
            let is_correct_gender = m.contains(gender_param) || (!m.contains("(3313202977=2204441813)") && !m.contains("(3313202977=3111576190)"));
            let is_correct_locale = m.to_lowercase().contains(&format!("={})", locale.to_lowercase())) || !m.contains("2441027675=");
            
            if is_correct_gender && is_correct_locale {
                sounds.push(m);
            }
        }
    }

    sounds.sort();
    sounds.dedup();

    // Generate SRT
    let srt_path = generate_srt(&app, &target_cg_name).unwrap_or_else(|e| {
        emit_log(&app, &format!("Warning: Could not generate subtitles: {}", e));
        None
    });

    Ok(VideoInfo {
        cg_name: target_cg_name,
        cg_file: mp4_path,
        girl_or_boy,
        sounds,
        srt_path,
    })
}

fn index_directory(root: &Path, ext: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut dirs_to_visit = vec![root.to_path_buf()];
    
    while let Some(dir) = dirs_to_visit.pop() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs_to_visit.push(path);
                } else if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                    result.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    result
}

// Minimal SRT generator
fn generate_srt(app: &AppHandle, cg_name: &str) -> Result<Option<String>, String> {
    let data_dir = get_data_dir(app);
    let videocaption_path = data_dir.join("videocaption.json");
    let multitext_path = data_dir.join("MultiText.json");
    
    if !videocaption_path.exists() || !multitext_path.exists() {
        return Ok(None);
    }

    let vc_str = fs::read_to_string(&videocaption_path).map_err(|e| e.to_string())?;
    let mt_str = fs::read_to_string(&multitext_path).map_err(|e| e.to_string())?;

    let videocaptions: Vec<Value> = serde_json::from_str(&vc_str).map_err(|e| e.to_string())?;
    let multitext: Vec<Value> = serde_json::from_str(&mt_str).map_err(|e| e.to_string())?;

    let mut dest_captions = Vec::new();
    for vc in videocaptions {
        if vc.get("CgName").and_then(|v| v.as_str()) == Some(cg_name) {
            dest_captions.push(vc);
        }
    }
    
    if dest_captions.is_empty() {
        return Ok(None);
    }
    
    dest_captions.sort_by_key(|c| c.get("CaptionId").and_then(|v| v.as_i64()).unwrap_or(0));

    let mut srt_content = String::new();
    let mut index = 1;

    for cap in dest_captions {
        let text_id = cap.get("CaptionText").and_then(|v| v.as_str()).unwrap_or("");
        let mut content = "";
        for mt in &multitext {
            if mt.get("Id").and_then(|v| v.as_str()) == Some(text_id) {
                content = mt.get("Content").and_then(|v| v.as_str()).unwrap_or("");
                break;
            }
        }
        
        if content.is_empty() { continue; }

        let show_moment = cap.get("ShowMoment").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let duration = cap.get("Duration").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let start_ms = (show_moment * (1000.0 / 30.0)) as i64;
        let end_ms = start_ms + (duration * (1000.0 / 30.0)) as i64;

        let start_str = format_srt_time(start_ms);
        let end_str = format_srt_time(end_ms);

        srt_content.push_str(&format!("{}\n{} --> {}\n{}\n\n", index, start_str, end_str, content));
        index += 1;
    }

    if srt_content.is_empty() {
        return Ok(None);
    }

    let out_dir = get_data_dir(app).join("captions");
    let _ = fs::create_dir_all(&out_dir);
    let srt_path = out_dir.join(format!("{}.srt", cg_name));
    fs::write(&srt_path, srt_content).map_err(|e| e.to_string())?;

    Ok(Some(srt_path.to_string_lossy().to_string()))
}

fn format_srt_time(ms: i64) -> String {
    let s = ms / 1000;
    let ms_part = ms % 1000;
    let m = s / 60;
    let s_part = s % 60;
    let h = m / 60;
    let m_part = m % 60;
    format!("{:02}:{:02}:{:02},{:03}", h, m_part, s_part, ms_part)
}

#[tauri::command]
pub fn get_video_audio_event(app: AppHandle, video_name: String) -> Result<Vec<String>, String> {
    let data_dir = get_data_dir(&app);
    let videodata_path = data_dir.join("videodata.json");
    let videosound_path = data_dir.join("videosound.json");

    if !videodata_path.exists() || !videosound_path.exists() {
        return Ok(vec![]);
    }

    let videodata_str = fs::read_to_string(&videodata_path).map_err(|e| e.to_string())?;
    let videosound_str = fs::read_to_string(&videosound_path).map_err(|e| e.to_string())?;

    let videodata: Vec<Value> = serde_json::from_str(&videodata_str).map_err(|e| e.to_string())?;
    let videosound: Vec<Value> = serde_json::from_str(&videosound_str).map_err(|e| e.to_string())?;

    let mp4 = PathBuf::from(video_name);
    let base_name = mp4.file_stem().unwrap().to_string_lossy().to_string().to_lowercase();
    
    let mut target_cg_name = String::new();
    for item in videodata {
        if let Some(cg_file) = item.get("CgFile").and_then(|v| v.as_str()) {
            let fn_name = cg_file.split('/').last().unwrap().split('.').next().unwrap().to_lowercase();
            if fn_name == base_name || base_name.contains(&fn_name) || fn_name.contains(&base_name) {
                target_cg_name = item.get("CgName").and_then(|v| v.as_str()).unwrap_or("").to_string();
                break;
            }
        }
    }

    if target_cg_name.is_empty() {
        return Ok(vec![]);
    }

    let mut events = Vec::new();
    for item in videosound {
        if let Some(cg_name) = item.get("CgName").and_then(|v| v.as_str()) {
            if cg_name == target_cg_name {
                if let Some(event_path) = item.get("EventPath").and_then(|v| v.as_str()) {
                    let event = event_path.split('/').last().unwrap().split('.').next().unwrap().to_string();
                    events.push(event);
                }
            }
        }
    }
    
    Ok(events)
}
