import os
import urllib.request
import zipfile
import shutil
from pathlib import Path

import sys

def get_base_dir():
    if getattr(sys, 'frozen', False):
        return Path(sys.executable).parent
    return Path(__file__).parent.parent

# Paths
BASE_DIR = get_base_dir()
TOOLS_DIR = BASE_DIR / "tools"
JSON_DIR = BASE_DIR / "data"
CAPTIONS_DIR = BASE_DIR / "Captions"
VIDEOS_DIR = BASE_DIR / "Videos"

import json

def get_latest_branch():
    try:
        req = urllib.request.Request("https://api.github.com/repos/Arikatsu/WutheringWaves_Data")
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode())
            return data.get("default_branch", "3.5")
    except:
        return "3.5"

FFMPEG_URL = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
WWISER_URL = "https://github.com/bnnm/wwiser/archive/refs/heads/master.zip"

def get_latest_vgmstream_url():
    try:
        req = urllib.request.Request("https://api.github.com/repos/vgmstream/vgmstream/releases/latest")
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode())
            for asset in data.get("assets", []):
                if asset.get("name") == "vgmstream-win64.zip":
                    return asset.get("browser_download_url")
    except:
        pass
    return "https://github.com/vgmstream/vgmstream/releases/download/r2117/vgmstream-win64.zip"

def download_file(url, dest_path, progress_callback=None):
    def reporthook(blocknum, blocksize, totalsize):
        if progress_callback and totalsize > 0:
            percent = min(1.0, (blocknum * blocksize) / totalsize)
            progress_callback(percent)
            
    urllib.request.urlretrieve(url, dest_path, reporthook)

def setup_tools(log_callback=None, progress_callback=None):
    TOOLS_DIR.mkdir(exist_ok=True)
    
    # FFmpeg
    ffmpeg_exe = TOOLS_DIR / "ffmpeg.exe"
    if not ffmpeg_exe.exists():
        zip_path = TOOLS_DIR / "ffmpeg.zip"
        if log_callback: log_callback("Downloading FFmpeg (takes a while)...")
        download_file(FFMPEG_URL, zip_path, progress_callback)
        if log_callback: log_callback("Extracting FFmpeg...")
        with zipfile.ZipFile(zip_path, 'r') as zip_ref:
            for file_info in zip_ref.infolist():
                if file_info.filename.endswith('ffmpeg.exe'):
                    file_info.filename = 'ffmpeg.exe'
                    zip_ref.extract(file_info, TOOLS_DIR)
        zip_path.unlink()
        
    # vgmstream
    vgmstream_exe = TOOLS_DIR / "vgmstream-cli.exe"
    if not vgmstream_exe.exists():
        zip_path = TOOLS_DIR / "vgmstream.zip"
        if log_callback: log_callback("Downloading vgmstream...")
        vgmstream_url = get_latest_vgmstream_url()
        download_file(vgmstream_url, zip_path, progress_callback)
        if log_callback: log_callback("Extracting vgmstream...")
        with zipfile.ZipFile(zip_path, 'r') as zip_ref:
            zip_ref.extractall(TOOLS_DIR)
        zip_path.unlink()

    # Wwiser
    wwiser_dir = TOOLS_DIR / "wwiser-master"
    wwiser_py = wwiser_dir / "wwiser.py"
    if not wwiser_py.exists():
        zip_path = TOOLS_DIR / "wwiser.zip"
        if log_callback: log_callback("Downloading Wwiser...")
        download_file(WWISER_URL, zip_path, progress_callback)
        if log_callback: log_callback("Extracting Wwiser...")
        with zipfile.ZipFile(zip_path, 'r') as zip_ref:
            zip_ref.extractall(TOOLS_DIR)
        zip_path.unlink()
        
    if log_callback: log_callback("Tools check completed.")

def setup_json_data(log_callback=None, force_update=False, textmap_lang="th", progress_callback=None):
    JSON_DIR.mkdir(exist_ok=True)
    CAPTIONS_DIR.mkdir(exist_ok=True)
    VIDEOS_DIR.mkdir(exist_ok=True)
    
    branch = get_latest_branch()
    if log_callback: log_callback(f"Using database version (branch): {branch}")
    
    json_urls = {
        "videodata.json": f"https://raw.githubusercontent.com/Arikatsu/WutheringWaves_Data/{branch}/BinData/cgVedio/videodata.json",
        "videosound.json": f"https://raw.githubusercontent.com/Arikatsu/WutheringWaves_Data/{branch}/BinData/cgVedio/videosound.json",
        "MultiText.json": f"https://raw.githubusercontent.com/Arikatsu/WutheringWaves_Data/{branch}/Textmaps/{textmap_lang}/multi_text/MultiText.json",
    }
    
    import shutil
    legacy_caption = BASE_DIR / "legacy" / "videocaption.json"
    data_caption = JSON_DIR / "videocaption.json"
    if legacy_caption.exists() and not data_caption.exists():
        shutil.copy(legacy_caption, data_caption)

    
    for name, url in json_urls.items():
        file_path = JSON_DIR / name
        if not file_path.exists() or force_update:
            if log_callback: log_callback(f"Downloading {name} ({textmap_lang})...")
            try:
                download_file(url, file_path, progress_callback)
            except Exception as e:
                if log_callback: log_callback(f"Failed to download {name}: {e}")
            
    if log_callback: log_callback("JSON Data check completed.")

def get_ffmpeg_path():
    return str(TOOLS_DIR / "ffmpeg.exe")

def get_vgmstream_path():
    return str(TOOLS_DIR / "vgmstream-cli.exe")

def get_wwiser_path():
    return str(TOOLS_DIR / "wwiser-master" / "wwiser.py")
