# Detailed Guide: How to use WuWa Cutscene Exporter

## 1. Unpack the Media Resources
You will need [FModel](https://github.com/4sval/FModel) to unpack the game files.
1. Run FModel and select the game folder (e.g. `C:\PathTo\Epic Games\WutheringWavesj3oFh\Wuthering Waves Game`).
2. Load the packs using the AES key.
3. If you want Japanese or other voice languages, make sure to change the **Voice Language** in FModel's settings BEFORE exporting!
4. Export the following folders (as raw files):
   - `Client/Content/Aki/Movies`
   - `Client/Content/Aki/WwiseAudio_Generated`

## 2. Using the Tool
Launch `WuWa_Cutscene_Exporter.exe` (or run `python main.py`).

### Step A: Setup & Tools (Settings Tab)
- Go to the **Settings ⚙️** tab.
- Select your desired **Subtitle Language** from the dropdown.
- Click **Force Update JSON Data** (This pulls the latest game text and cutscene mappings from GitHub).
- Click **Download Missing Tools** (This will automatically download FFmpeg, vgmstream, and wwiser in the background).

### Step B: Extract Audio (Wwiser Tab)
- Go to the **Wwiser 🎙️** tab.
- Click **Browse** and select the `WwiseAudio_Generated` folder you exported from FModel.
- Click **Extract Audio (Run Wwiser)**. The tool will scan all `.bnk` files and extract them to `.txtp` format automatically. Wait for the progress bar to finish.

### Step C: Render Videos (Video Renderer Tab)
- Go to the **Video Renderer 🎬** tab.
- Click **Browse** and select the `Movies` folder you exported from FModel.
- Choose whether you want the **Rover (Girl)** or **Rover (Boy)** cutscenes (or **Both**).
- Enter the **Voice Locale** you exported (e.g., `ja` for Japanese, `en` for English).
- Select the **Subtitle Mode**:
  - `None`: No subtitles.
  - `Soft-sub`: Embeds subtitles as a toggleable track (best for MKV/MP4 players).
  - `Hard-sub`: Burns subtitles directly into the video (using Kanit Medium font).
- Custom BGM (Optional): If a cutscene is missing music (e.g. boss fights where music carries over from gameplay), you can click Browse to select an `.mp3`, `.wav`, or `.txtp` file to automatically mix into the final video.
- Click **Start Video Rendering**. 

The tool will now pair the audio, generate the `.srt` files, and use FFmpeg to mux them into the final `.mp4` files. You can watch the real-time log to track the progress!

## Building the EXE yourself
If you modified the code and want to package it into a new `.exe`:
```powershell
pip install -r requirements.txt
python build.py
```
The new executable will be inside the `dist/` folder.
