# WuWa Cutscene Exporter (All-in-One Premium)

> [!WARNING]  
> Movies (*.mp4) in 2.1 and after are encrypted. Use the latest version of [4sval/FModel](https://github.com/4sval/FModel) to export videos.

A premium, fully automated tool to export Wuthering Waves cutscenes/movies with embedded subtitles and synchronized audio.

## Features
- **All-in-One GUI:** Easy-to-use interface with tabs and real-time logs.
- **Auto Dependency Management:** Automatically downloads FFmpeg, vgmstream, and wwiser. No more manual scoop installations!
- **Dynamic Database Updates:** Automatically fetches the latest JSON data (videodata, MultiText, etc.) from [Arikatsu's Repo](https://github.com/Arikatsu/WutheringWaves_Data).
- **Subtitle Selection:** Choose your preferred subtitle language (en, ja, th, etc.) directly in the UI.
- **Wwiser Automation:** Simply point to the `WwiseAudio_Generated` folder and the tool handles `.bnk` extraction automatically.

## How to Run

### Option 1: Use the Executable (Easiest)
Download or build the `WuWa_Cutscene_Exporter.exe` from the `dist/` folder. Double-click it to run the GUI immediately. No Python installation required!

### Option 2: Run from Source
If you prefer to run the Python scripts directly:
```powershell
pip install -r requirements.txt
python main.py
```

## Usage
1. Unpack the game's `Client/Content/Aki/Movies` and `Client/Content/Aki/WwiseAudio_Generated` folders using FModel.
2. Open **WuWa Cutscene Exporter**.
3. In the **Settings** tab, choose your preferred Subtitle Language and click "Download Missing Tools".
4. In the **Wwiser** tab, select the `WwiseAudio_Generated` folder and click "Extract Audio".
5. In the **Video Renderer** tab, select the `Movies` folder, choose Boy/Girl Rover, choose voice locale (e.g. `ja` for Japanese), and click "Start Video Rendering".
6. Check the `Videos/` folder for your finished cutscenes!

**For more detailed instructions, see [HOW_TO.md](HOW_TO.md).**

## Credits & Acknowledgements
This project wouldn't be possible without the amazing work of the community:
- **[SunsetMkt](https://github.com/SunsetMkt/WuWa-Cutscenes)**: For creating the original base version of this tool, which this project was forked and evolved from.
- **[Arikatsu](https://github.com/Arikatsu)**: For the [WutheringWaves_Data](https://github.com/Arikatsu/WutheringWaves_Data) repository, providing the essential Datamined JSONs (videodata, videosound, MultiText).
- **[bnnm](https://github.com/bnnm)**: For the incredible [wwiser](https://github.com/bnnm/wwiser) audio parser which makes it possible to reconstruct Wwise audio banks.
- **[vgmstream team](https://github.com/vgmstream/vgmstream)**: For the robust audio decoding library used to convert game audio streams.
- **[BtbN](https://github.com/BtbN)**: For providing compiled [FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds) used by the renderer.
