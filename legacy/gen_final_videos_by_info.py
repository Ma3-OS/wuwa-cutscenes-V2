import json
import os
import shutil
import subprocess

import argparse

# Default configuration
GirlOnly = True  # Only generate Girl version


def load_json(abspath):
    with open(abspath, "r", encoding="utf-8") as f:
        return json.load(f)


def save_json(abspath, data):
    with open(abspath, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=4)


def touch_dir(dir_path):
    os.makedirs(dir_path, exist_ok=True)


def call_vgmstream(infile, outfile):
    subprocess.call(
        [
            "vgmstream-cli",
            "-o",  # <outfile.wav>: name of output .wav file
            outfile,
            infile,
        ]
    )


def get_filename(path):
    return os.path.basename(path)


def get_filename_no_ext(path):
    return os.path.splitext(get_filename(path))[0]


def copy_to(src, dst, overwrite=True):
    if os.path.exists(dst) and overwrite:
        os.remove(dst)
    path = shutil.copyfile(src, dst)
    return path


def move_to(src, dst, overwrite=True):
    if os.path.exists(dst) and overwrite:
        os.remove(dst)
    path = shutil.move(src, dst)
    return path


def get_abs_path(rel_path):
    script_path = os.path.dirname(os.path.realpath(__file__))
    full_path = os.path.join(script_path, rel_path)
    norm_path = os.path.normpath(full_path)
    return norm_path


def call_ffmpeg(cmd):
    subprocess.call(["ffmpeg"] + cmd)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate final videos")
    parser.add_argument("--both", action="store_true", help="Generate both Girl and Boy versions (Default: Girl only)")
    args = parser.parse_args()
    GirlOnly = not args.both

    videos_info = load_json("videos_info.json")
    touch_dir("Videos")  # To save final videos
    touch_dir("Sounds")  # To save soundtracks

    # GirlOnly
    if GirlOnly:
        videos_info = [item for item in videos_info if item["GirlOrBoy"] == "Girl"]

    # Phase 1: Generate all soundtracks regardless of videos
    print("Generating soundtracks")
    for video in videos_info:
        new_sounds = []
        for sound in video["Sound"]:
            print(f"Generating {sound}")
            wav_name = get_filename_no_ext(sound) + ".wav"
            wav_path = os.path.join("Sounds", wav_name)

            # If wav file already exists, skip
            if not os.path.exists(wav_path):
                call_vgmstream(sound, wav_name)
                # NOTE: vgmstream-cli could only generate wav outfile to current working directory.
                # So we need to move outfile to Sounds folder
                move_to(wav_name, wav_path)

            new_sounds.append(get_abs_path(wav_path))
        video["Sound"] = new_sounds

    # Phase 2: Generate final videos
    print("Generating final videos")
    for video in videos_info:
        video_name = (
            f"{video['CgName']}_{video['GirlOrBoy']}_{get_filename_no_ext(video['CgFile'])}"
            + ".mp4"
        )
        print(f"Generating {video_name}")

        # Input video and audio
        video_file = video["CgFile"]
        audio_files = video["Sound"]
        out_path = os.path.join("Videos", video_name)

        cmd = ["-y", "-i", video_file]

        audio_count = len(audio_files)

        # Check if subtitle exists
        srt_path = os.path.join("Captions", f"{video['CgName']}.srt")
        has_srt = os.path.exists(srt_path)

        # If no audio
        if audio_count == 0:
            if has_srt:
                cmd += ["-i", srt_path, "-map", "0:v:0", "-map", "1:s:0", "-c", "copy", "-c:s", "mov_text", out_path]
            else:
                cmd += ["-c", "copy", out_path]

        # If single audio
        elif audio_count == 1:
            cmd += ["-i", audio_files[0]]
            if has_srt:
                cmd += ["-i", srt_path]
            
            cmd += [
                "-filter_complex",
                "aformat=channel_layouts=stereo[aout]",
                "-map",
                "0:v:0",
                "-map",
                "[aout]"
            ]
            
            if has_srt:
                cmd += ["-map", "2:s:0", "-c:s", "mov_text"]
                
            cmd += [
                "-c:v",
                "copy",
                "-c:a",
                "aac",
                "-shortest",
                out_path,
            ]

        # If multiple audio, mix
        else:
            for a in audio_files:
                cmd += ["-i", a]
                
            if has_srt:
                cmd += ["-i", srt_path]
                srt_index = 1 + audio_count

            amix_inputs = "".join(f"[{i + 1}:a]" for i in range(audio_count))
            filter_complex = (
                f"{amix_inputs}"
                f"amix=inputs={audio_count}:duration=longest,"
                f"aresample=async=1,"
                f"aformat=channel_layouts=stereo[aout]"
            )

            cmd += [
                "-filter_complex",
                filter_complex,
                "-map",
                "0:v:0",
                "-map",
                "[aout]"
            ]
            
            if has_srt:
                cmd += ["-map", f"{srt_index}:s:0", "-c:s", "mov_text"]
                
            cmd += [
                "-c:v",
                "copy",
                "-c:a",
                "aac",
                "-shortest",
                out_path,
            ]

        call_ffmpeg(cmd)
