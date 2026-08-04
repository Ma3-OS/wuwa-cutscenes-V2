import os
import subprocess
from pathlib import Path

from core.downloader import VIDEOS_DIR, CAPTIONS_DIR, get_ffmpeg_path, get_vgmstream_path

def get_filename_no_ext(path):
    return os.path.splitext(os.path.basename(path))[0]

def call_vgmstream(infile, outfile, log_callback=None):
    vgmstream_exe = get_vgmstream_path()
    cmd = [vgmstream_exe, "-o", outfile, infile]
    
    if log_callback:
        process = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1, universal_newlines=True)
        for line in iter(process.stdout.readline, ''):
            if line: log_callback("vgmstream: " + line.strip())
        process.wait()
    else:
        subprocess.call(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

import concurrent.futures
from threading import Lock

def render_single_video(i, video, total, subtitle_mode, log_callback, progress_callback):
    # 1. Generate soundtracks
    new_sounds = []
    for sound in video["Sound"]:
        wav_name = get_filename_no_ext(sound) + ".wav"
        wav_path = VIDEOS_DIR / wav_name
        
        if not wav_path.exists():
            call_vgmstream(sound, str(wav_path), None) # Mute vgmstream logs in threads to avoid clutter
            
        if wav_path.exists():
            new_sounds.append(str(wav_path))
        else:
            if log_callback: log_callback(f"WRN: {wav_name} was not generated. Skipping this track.")
            
    video["Sound"] = new_sounds
    
    # 2. Render final video
    video_name = f"{video['CgName']}_{video['GirlOrBoy']}_{get_filename_no_ext(video['CgFile'])}.mp4"
    out_path = VIDEOS_DIR / video_name
    
    if out_path.exists():
        if progress_callback: progress_callback(1)
        return True
        
    video_file = video["CgFile"]
    audio_files = video["Sound"]
    audio_count = len(audio_files)
    
    srt_path = CAPTIONS_DIR / f"{video['CgName']}.srt"
    has_srt = (subtitle_mode != "None") and srt_path.exists()
    is_hardsub = has_srt and (subtitle_mode == "Hard-sub")
    is_softsub = has_srt and (subtitle_mode == "Soft-sub")
    
    ffmpeg_exe = get_ffmpeg_path()
    cmd = [ffmpeg_exe, "-y", "-i", video_file]
    
    if audio_count == 0:
        if is_softsub:
            cmd += ["-i", str(srt_path), "-map", "0:v:0", "-map", "1:s:0", "-c", "copy", "-c:s", "mov_text", str(out_path)]
        elif is_hardsub:
            # Escape path for video filter: replace backward slashes with forward slashes and escape colon
            escaped_srt = str(srt_path).replace('\\', '/').replace(':', '\\:')
            cmd += ["-vf", f"subtitles='{escaped_srt}':force_style='Fontname=Kanit Medium,Outline=1,Shadow=0'", "-map", "0:v:0", "-c:v", "libx264", str(out_path)]
        else:
            cmd += ["-c", "copy", str(out_path)]
            
    elif audio_count == 1:
        cmd += ["-i", audio_files[0]]
        if is_softsub: cmd += ["-i", str(srt_path)]
        
        cmd += ["-filter_complex", "aformat=channel_layouts=stereo[aout]", "-map", "0:v:0", "-map", "[aout]"]
        
        if is_softsub: 
            cmd += ["-map", "2:s:0", "-c:s", "mov_text"]
            
        if is_hardsub:
            escaped_srt = str(srt_path).replace('\\', '/').replace(':', '\\:')
            cmd += ["-vf", f"subtitles='{escaped_srt}':force_style='Fontname=Kanit Medium,Outline=1,Shadow=0'", "-c:v", "libx264", "-c:a", "aac"]
        else:
            cmd += ["-c:v", "copy", "-c:a", "aac"]
            
        if not has_srt: cmd += ["-shortest"]
        cmd += [str(out_path)]
        
    else:
        for a in audio_files:
            cmd += ["-i", a]
        if is_softsub:
            cmd += ["-i", str(srt_path)]
            srt_index = 1 + audio_count
            
        amix_inputs = "".join(f"[{j + 1}:a]" for j in range(audio_count))
        filter_complex = f"{amix_inputs}amix=inputs={audio_count}:duration=longest,aresample=async=1,aformat=channel_layouts=stereo[aout]"
        
        if is_hardsub:
            escaped_srt = str(srt_path).replace('\\', '/').replace(':', '\\:')
            filter_complex = f"subtitles='{escaped_srt}':force_style='Fontname=Kanit Medium,Outline=2,Shadow=0';{filter_complex}"
            
        cmd += ["-filter_complex", filter_complex, "-map", "0:v:0", "-map", "[aout]"]
        
        if is_softsub: 
            cmd += ["-map", f"{srt_index}:s:0", "-c:s", "mov_text"]
            
        if is_hardsub:
            cmd += ["-c:v", "libx264", "-c:a", "aac"]
        else:
            cmd += ["-c:v", "copy", "-c:a", "aac"]
            
        if not has_srt: cmd += ["-shortest"]
        cmd += [str(out_path)]
        
    if log_callback: log_callback(f"Rendering {video_name}...")
    
    process = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, bufsize=1, universal_newlines=True)
    error_lines = []
    for line in iter(process.stderr.readline, ''):
        if line:
            if not ("time=" in line or "frame=" in line):
                error_lines.append(line.strip('\r\n'))
                
    return_code = process.wait()
    
    if return_code != 0:
        if log_callback: 
            log_callback(f"ERR: ffmpeg failed for {video_name}")
            for err in error_lines[-5:]:
                log_callback(f"ffmpeg err: {err}")
    else:
        if log_callback: log_callback(f"Finished {video_name}")
                
    if progress_callback: progress_callback(1)
    return True

def render_videos(videos_info, subtitle_mode="Soft-sub", custom_bgm="", log_callback=None, progress_callback=None):
    if log_callback: log_callback(f"Starting Multi-threaded Render (Mode: {subtitle_mode})...")
    
    total = len(videos_info)
    completed = 0
    lock = Lock()
    
    def locked_progress(step):
        nonlocal completed
        with lock:
            completed += step
            if progress_callback: progress_callback(completed / total)

    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
        futures = []
        for i, video in enumerate(videos_info):
            # Inject custom BGM if provided
            if custom_bgm and os.path.exists(custom_bgm):
                video["Sound"].append(custom_bgm)
                
            futures.append(executor.submit(render_single_video, i, video, total, subtitle_mode, log_callback, locked_progress))
        
        for future in concurrent.futures.as_completed(futures):
            future.result()
            
    # Clean up temporary .wav files
    if log_callback: log_callback("Cleaning up temporary audio files...")
    for f in os.listdir(VIDEOS_DIR):
        if f.endswith('.wav'):
            try:
                os.remove(VIDEOS_DIR / f)
            except Exception as e:
                pass
                
    if log_callback: log_callback("All videos rendered successfully!")
    return True
