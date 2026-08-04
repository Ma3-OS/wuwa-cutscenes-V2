use std::fs;
use std::path::{PathBuf};
use std::process::Command;
use tauri::AppHandle;

use crate::commands::downloader::get_tools_dir;
use crate::commands::generator::VideoInfo;
use crate::commands::log::emit_log;

#[tauri::command]
pub async fn render_video(
    app: AppHandle,
    video_info: VideoInfo,
    subtitle_mode: String,
) -> Result<(), String> {
    let tools_dir = get_tools_dir(&app);
    let vgmstream_exe = tools_dir.join("vgmstream-cli.exe");
    let ffmpeg_exe = tools_dir.join("ffmpeg.exe");

    if !vgmstream_exe.exists() { return Err("vgmstream-cli.exe not found".into()); }
    if !ffmpeg_exe.exists() { return Err("ffmpeg.exe not found".into()); }

    let output_dir = crate::commands::downloader::get_data_dir(&app).join("output");
    let _ = fs::create_dir_all(&output_dir);

    // 1. Convert txtp to wav
    let mut wav_paths = Vec::new();
    for sound in &video_info.sounds {
        let sound_path = PathBuf::from(sound);
        let wav_name = format!("{}.wav", sound_path.file_stem().unwrap().to_string_lossy());
        let wav_path = output_dir.join(&wav_name);
        
        if !wav_path.exists() {
            emit_log(&app, &format!("Converting {} to wav...", sound_path.file_name().unwrap().to_string_lossy()));
            let status = Command::new(&vgmstream_exe)
                .args(["-o", &wav_path.to_string_lossy(), &sound_path.to_string_lossy()])
                .status()
                .map_err(|e| e.to_string())?;
                
            if !status.success() {
                emit_log(&app, &format!("Warning: vgmstream failed for {}", sound));
            }
        }
        
        if wav_path.exists() {
            wav_paths.push(wav_path);
        }
    }

    // 2. FFmpeg Render
    let video_path = PathBuf::from(&video_info.cg_file);
    let out_name = format!("{}_{}_{}.mp4", video_info.cg_name, video_info.girl_or_boy, video_path.file_stem().unwrap().to_string_lossy());
    let out_path = output_dir.join(&out_name);

    if out_path.exists() {
        emit_log(&app, &format!("Video {} already exists. Skipping.", out_name));
        return Ok(());
    }

    emit_log(&app, &format!("Rendering {}...", out_name));

    let mut cmd = Command::new(&ffmpeg_exe);
    cmd.args(["-y", "-i", &video_info.cg_file]);

    let audio_count = wav_paths.len();
    for wav in &wav_paths {
        cmd.args(["-i", &wav.to_string_lossy()]);
    }

    let has_srt = subtitle_mode != "None" && video_info.srt_path.is_some();
    let is_hardsub = has_srt && subtitle_mode == "Hard-sub";
    let is_softsub = has_srt && subtitle_mode == "Soft-sub";
    
    let srt_path = video_info.srt_path.unwrap_or_default();
    let mut srt_index = 1 + audio_count;

    if is_softsub {
        cmd.args(["-i", &srt_path]);
    }

    if audio_count == 0 {
        if is_softsub {
            cmd.args(["-map", "0:v:0", "-map", "1:s:0", "-c", "copy", "-c:s", "mov_text"]);
        } else if is_hardsub {
            let escaped_srt = srt_path.replace("\\", "/").replace(":", "\\:");
            cmd.args(["-vf", &format!("subtitles='{}':force_style='Fontname=Kanit Medium,Outline=1,Shadow=0'", escaped_srt), "-map", "0:v:0", "-c:v", "libx264"]);
        } else {
            cmd.args(["-c", "copy"]);
        }
    } else if audio_count == 1 {
        cmd.args(["-filter_complex", "aformat=channel_layouts=stereo[aout]", "-map", "0:v:0", "-map", "[aout]"]);
        
        if is_softsub {
            cmd.args(["-map", &format!("{}:s:0", srt_index), "-c:s", "mov_text"]);
        }
        if is_hardsub {
            let escaped_srt = srt_path.replace("\\", "/").replace(":", "\\:");
            cmd.args(["-vf", &format!("subtitles='{}':force_style='Fontname=Kanit Medium,Outline=1,Shadow=0'", escaped_srt), "-c:v", "libx264", "-c:a", "aac"]);
        } else {
            cmd.args(["-c:v", "copy", "-c:a", "aac"]);
        }
        if !has_srt { cmd.arg("-shortest"); }
    } else {
        let mut amix_inputs = String::new();
        for j in 0..audio_count {
            amix_inputs.push_str(&format!("[{}:a]", j + 1));
        }
        let mut filter_complex = format!("{}amix=inputs={}:duration=longest,aresample=async=1,aformat=channel_layouts=stereo[aout]", amix_inputs, audio_count);
        
        if is_hardsub {
            let escaped_srt = srt_path.replace("\\", "/").replace(":", "\\:");
            filter_complex = format!("subtitles='{}':force_style='Fontname=Kanit Medium,Outline=2,Shadow=0';{}", escaped_srt, filter_complex);
        }
        
        cmd.args(["-filter_complex", &filter_complex, "-map", "0:v:0", "-map", "[aout]"]);
        
        if is_softsub {
            cmd.args(["-map", &format!("{}:s:0", srt_index), "-c:s", "mov_text"]);
        }
        if is_hardsub {
            cmd.args(["-c:v", "libx264", "-c:a", "aac"]);
        } else {
            cmd.args(["-c:v", "copy", "-c:a", "aac"]);
        }
        if !has_srt { cmd.arg("-shortest"); }
    }

    cmd.arg(&out_path);

    let output = cmd.output().map_err(|e| e.to_string())?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        emit_log(&app, &format!("FFmpeg error: {}", stderr));
        return Err("Video rendering failed".into());
    }

    emit_log(&app, &format!("Finished {}", out_name));

    // Cleanup temporary wav files
    for wav in wav_paths {
        let _ = fs::remove_file(wav);
    }

    Ok(())
}
