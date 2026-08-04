use std::fs;
use std::path::{PathBuf};
use std::process::Command;
use tauri::AppHandle;

use crate::config::ConfigState;
use crate::commands::downloader::get_tools_dir;
use crate::commands::log::emit_log;

#[tauri::command]
pub async fn run_wwiser(app: AppHandle, state: tauri::State<'_, ConfigState>, target_files: Option<Vec<String>>) -> Result<(), String> {
    let data_dir = crate::commands::downloader::get_data_dir(&app);
    let wwise_path = data_dir.join("wwise_audio");
    
    // If no specific files are provided, we need a valid wwise_path
    if target_files.is_none() && !wwise_path.exists() {
        return Err("Audio banks have not been extracted from paks yet.".to_string());
    }
    
    let tools_dir = get_tools_dir(&app);
    let wwiser_py = tools_dir.join("wwiser-master").join("wwiser.py");
    if !wwiser_py.exists() {
        return Err("wwiser.py not found. Please download tools first.".to_string());
    }
    
    // Fast check: if txtp folder exists, we can skip running wwiser
    let txtp_out_dir = wwise_path.join("txtp");
    let has_txtp = txtp_out_dir.exists() && txtp_out_dir.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false);
    
    if has_txtp {
        emit_log(&app, "Audio banks already extracted (.txtp found). Skipping Wwiser.");
        return Ok(());
    }

    emit_log(&app, "Scanning for .bnk files...");
    
    // Flatten Media folder only if we are doing a full scan
    if target_files.is_none() {
        let media_dir = wwise_path.join("Media");
        if media_dir.exists() {
            emit_log(&app, "Flattening localized Media files...");
            if let Ok(entries) = fs::read_dir(&media_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(sub_entries) = fs::read_dir(&path) {
                            for sub_entry in sub_entries.flatten() {
                                let sub_path = sub_entry.path();
                                if sub_path.extension().and_then(|s| s.to_str()) == Some("wem") {
                                    let dst = media_dir.join(sub_path.file_name().unwrap());
                                    if !dst.exists() {
                                        let _ = fs::copy(&sub_path, &dst);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Find all .bnk
    let mut bnk_files = Vec::new();
    if let Some(files) = target_files {
        for f in files {
            bnk_files.push(PathBuf::from(f));
        }
    } else {
        let mut dirs_to_visit = vec![wwise_path.clone()];
        while let Some(dir) = dirs_to_visit.pop() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        dirs_to_visit.push(path);
                    } else if path.extension().and_then(|s| s.to_str()) == Some("bnk") {
                        bnk_files.push(path);
                    }
                }
            }
        }
    }

    if bnk_files.is_empty() {
        return Err("No .bnk files found.".to_string());
    }

    let num_threads = 4;
    let chunk_size = std::cmp::max(1, bnk_files.len() / num_threads);
    let chunks: Vec<Vec<PathBuf>> = bnk_files.chunks(chunk_size).map(|c| c.to_vec()).collect();

    emit_log(&app, &format!("Found {} .bnk files. Running {} parallel wwiser workers to speed up extraction...", bnk_files.len(), chunks.len()));

    let txtp_dir = wwise_path.join("txtp"); // Output to txtp subfolder
    let _ = fs::create_dir_all(&txtp_dir);

    let mut config_paths = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let config_path = txtp_dir.join(format!("wwconfig_args_{}.txt", i));
        config_paths.push(config_path.clone());
        let mut config_content = String::new();
        config_content.push_str("-g\n-go\n");
        config_content.push_str(&format!("\"{}\"\n", txtp_dir.display().to_string().replace("\\", "/")));
        config_content.push_str("-gw\n\"../Media\"\n");
        for bnk in chunk {
            config_content.push_str(&format!("\"{}\"\n", bnk.display().to_string().replace("\\", "/")));
        }
        fs::write(&config_path, config_content).map_err(|e| e.to_string())?;
    }

    let total_banks = bnk_files.len();
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    let running = Arc::new(AtomicBool::new(true));
    let r_clone = running.clone();
    let app_clone = app.clone();
    let txtp_dir_clone = txtp_dir.clone();

    std::thread::spawn(move || {
        let mut last_count = 0;
        let mut seconds = 0;
        while r_clone.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_secs(2));
            if r_clone.load(Ordering::Relaxed) && txtp_dir_clone.exists() {
                seconds += 2;
                let mut count = 0;
                if let Ok(entries) = fs::read_dir(&txtp_dir_clone) {
                    for entry in entries.flatten() {
                        if entry.path().extension().and_then(|s| s.to_str()) == Some("txtp") {
                            count += 1;
                        }
                    }
                }
                if count != last_count {
                    last_count = count;
                    emit_log(&app_clone, &format!("Extracting audio banks... ({}/{} files generated)", count, total_banks));
                } else if count == 0 {
                    // Still parsing, notify the user so they know it isn't completely frozen
                    if seconds % 10 == 0 {
                        emit_log(&app_clone, &format!("Wwiser is parsing audio banks... ({}s elapsed, 0 files generated yet)", seconds));
                    }
                }
            }
        }
    });

    let mut children = Vec::new();
    for config_path in &config_paths {
        let child = Command::new("python")
            .arg(&wwiser_py)
            .arg(format!("--config={}", config_path.display()))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| {
                running.store(false, Ordering::Relaxed);
                format!("Failed to spawn python worker: {}", e)
            })?;
        children.push(child);
    }

    let mut success = true;
    for mut child in children {
        if let Ok(status) = child.wait() {
            if !status.success() {
                success = false;
            }
        } else {
            success = false;
        }
    }

    running.store(false, Ordering::Relaxed);
    for config_path in &config_paths {
        let _ = fs::remove_file(config_path);
    }

    if !success {
        return Err("One or more Wwiser parallel workers failed.".to_string());
    }

    emit_log(&app, "Extraction completed successfully.");
    Ok(())
}
