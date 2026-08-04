import json
import os
import re
import copy
from pathlib import Path
import pysrt

from core.downloader import JSON_DIR, CAPTIONS_DIR

def load_json(filename):
    path = JSON_DIR / filename
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)

def save_json(filename, data):
    path = JSON_DIR / filename
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=4)

def frame_to_ms(frame, fps=30):
    return frame * (1000 / fps)

def generate_captions(log_callback=None):
    if log_callback: log_callback("Generating subtitles...")
    
    try:
        VideoCaption = load_json("videocaption.json")
        MultiText = load_json("MultiText.json")
    except FileNotFoundError:
        if log_callback: log_callback("ERR: JSON data missing. Please update database.")
        return False

    MultiTextDict = {item["Id"]: item["Content"] for item in MultiText}
    
    CgNameList = list(dict.fromkeys([Cg["CgName"] for Cg in VideoCaption if "CgName" in Cg]))
    
    if not CgNameList:
        if log_callback: log_callback("WARN: 'CgName' not found in videocaption.json (file might be malformed). Skipping subtitle generation.")
        return False
    
    for CgName in CgNameList:
        DestCaption = [c for c in VideoCaption if c["CgName"] == CgName]
        DestCaption.sort(key=lambda x: x["CaptionId"])
        
        new_srt = pysrt.SubRipFile()
        for count, caption in enumerate(DestCaption):
            text = MultiTextDict.get(caption["CaptionText"], "")
            if not text: continue
            
            start_ms = frame_to_ms(caption.get("ShowMoment", 0))
            end_ms = frame_to_ms(caption.get("ShowMoment", 0) + caption.get("Duration", 0))
            
            new_srt.append(
                pysrt.SubRipItem(
                    index=count,
                    start=pysrt.SubRipTime(milliseconds=start_ms),
                    end=pysrt.SubRipTime(milliseconds=end_ms),
                    text=text,
                )
            )
            
        srt_path = CAPTIONS_DIR / f"{CgName}.srt"
        new_srt.save(str(srt_path))
        
    if log_callback: log_callback(f"Generated {len(CgNameList)} subtitle files.")
    return True

# ----------- Video Info Generator -----------

def _index_directory(root_dir):
    result = []
    for root, dirs, files in os.walk(root_dir):
        for file in files:
            result.append(os.path.join(root, file))
    return result

def search_all_files(files_cache, target_name):
    target_name = target_name.lower()
    exact_results = [path for path in files_cache if target_name == os.path.splitext(os.path.basename(path))[0].lower()]
    if exact_results:
        return exact_results
    results = [path for path in files_cache if target_name in os.path.basename(path).lower()]
    return results

def get_filename_by_CgFile(CgFile):
    fixup_map = {
        "M0206_Mp4": "M0206_Nanzhu",
        "M0206_nvzhu_Mp4": "M0206_Nvzhu",
        "DaPaoPoJieJIe_Mp4": "DaPaoPoJieJie",
        "M2_12_20_Nan": "M2_12_20_Seq_Nan",
        "M2_12_20_Nv": "M2_12_20_Seq_Nv",
        "M2_12_23_Nvzhu": "M2_12_23_Nv",
        "M2_12_23_Nanzhu": "M2_12_23_Nan",
    }
    fn = CgFile.split("/")[-1].split(".")[0]
    return fixup_map.get(fn, fn)

def get_path_by_CgFile(CgFile, movies_cache):
    fn = get_filename_by_CgFile(CgFile)
    results = search_all_files(movies_cache, fn)
    return results[0] if results else None

def get_events_by_CgName(CgName, videosound):
    items = [i for i in videosound if i["CgName"] == CgName]
    events = []
    fixup_map = {
        "play_story_music_3_0_b_m3_1_11_c": "play_story_music_3_0_b_m3_1_11 (4135626798=84696444)"
    }
    CgName_append_map = {
        "M3_2_38": ["play_story_music_m3_2_38"],
        "M3_0_28": ["play_story_music_3_0_a_m3_0_28"],
        "M3_2_13": ["play_story_music_m3_2_13"],
        "M3_10_16_1": ["play_sfx_lvb_m3_10_16_1"],
        "M0374": ["play_all_sound_m0374"],
    }

    for i in items:
        event = i["EventPath"].split("/")[-1].split(".")[0]
        events.append(fixup_map.get(event, event))

    if CgName in CgName_append_map:
        events.extend(CgName_append_map[CgName])

    return list(dict.fromkeys(events))

def get_all_sounds_by_CgName(CgName, GirlOrBoy, events, txtp_cache, locale):
    files = []
    param_map = {0: "(3313202977=2204441813)", 1: "(3313202977=3111576190)"}
    
    for event in events:
        this_event_files = []
        txtp_list = search_all_files(txtp_cache, event)
        if not txtp_list: continue

        if len(txtp_list) == 1:
            this_event_files.append(txtp_list[0])
            files.extend(this_event_files)
            continue

        if len(txtp_list) == 2:
            for txtp in txtp_list:
                if param_map[GirlOrBoy] in txtp:
                    this_event_files.append(txtp)
                    files.extend(this_event_files)
                    break

        if len(this_event_files) == 0 or len(txtp_list) > 2:
            for txtp in txtp_list:
                if param_map[int(not GirlOrBoy)] in txtp: continue
                this_event_files.append(txtp)
                files.extend(this_event_files)

    files = list(dict.fromkeys(files))

    # Locale filter
    locale_filtered_files = []
    for i in files:
        match = re.search(r"\(2441027675=([a-zA-Z]{2})\)", i)
        if match:
            if match.group(1).lower() == locale.lower():
                locale_filtered_files.append(i)
        else:
            locale_filtered_files.append(i)

    files = locale_filtered_files

    active_drop_words = ["_loading"]
    for word in active_drop_words:
        if not any(word in event for event in events):
            files = [i for i in files if word not in i]

    CgName_drop_map = {
        "M0362": ["(146205860=84696445) {m}", "(146205860=84696446) {m}"]
    }
    if CgName in CgName_drop_map:
        drop_words = CgName_drop_map[CgName]
        files = [i for i in files if not any(word in i for word in drop_words)]

    return files

def generate_videos_info(movies_path, txtp_path, locale, log_callback=None):
    if log_callback: log_callback("Matching videos and sounds...")
    
    try:
        videodata = load_json("videodata.json")
        videosound = load_json("videosound.json")
    except FileNotFoundError:
        if log_callback: log_callback("ERR: JSON data missing.")
        return False
        
    movies_cache = _index_directory(movies_path)
    txtp_cache = _index_directory(txtp_path)
    
    videos_info = []
    char_map = {0: "Girl", 1: "Boy"}
    
    for item in videodata:
        video = {
            "CgName": item["CgName"],
            "GirlOrBoy": char_map.get(item["GirlOrBoy"], "Girl"),
        }
        
        cg_file_path = get_path_by_CgFile(item["CgFile"], movies_cache)
        if not cg_file_path: continue
        
        video["CgFile"] = cg_file_path
        events = get_events_by_CgName(item["CgName"], videosound)
        video["Sound"] = get_all_sounds_by_CgName(item["CgName"], item["GirlOrBoy"], events, txtp_cache, locale)
        
        videos_info.append(video)

    save_json("videos_info.json", videos_info)
    if log_callback: log_callback(f"Successfully matched {len(videos_info)} videos.")
    return True
