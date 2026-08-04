import json
import os
from pathlib import Path

import sys

def get_base_dir():
    if getattr(sys, 'frozen', False):
        return Path(sys.executable).parent
    return Path(__file__).parent.parent

BASE_DIR = get_base_dir()
CONFIG_PATH = BASE_DIR / "config.json"

DEFAULT_CONFIG = {
    "movies_path": "",
    "wwise_path": "",
    "char_selection": "Girl",
    "locale_selection": "ja",
    "subtitle_mode": "Soft-sub",
}

class ConfigManager:
    def __init__(self):
        self.config = self.load()

    def load(self):
        if not CONFIG_PATH.exists():
            return DEFAULT_CONFIG.copy()
        
        try:
            with open(CONFIG_PATH, "r", encoding="utf-8") as f:
                loaded = json.load(f)
                # Ensure all default keys exist
                for k, v in DEFAULT_CONFIG.items():
                    if k not in loaded:
                        loaded[k] = v
                return loaded
        except:
            return DEFAULT_CONFIG.copy()

    def save(self):
        try:
            with open(CONFIG_PATH, "w", encoding="utf-8") as f:
                json.dump(self.config, f, indent=4)
        except Exception as e:
            print(f"Failed to save config: {e}")

    def get(self, key):
        return self.config.get(key, DEFAULT_CONFIG.get(key))

    def set(self, key, value):
        self.config[key] = value
        self.save()
