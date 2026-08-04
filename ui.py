import customtkinter as ctk
from tkinter import filedialog
import threading
import sys
import os

from core.downloader import setup_tools, setup_json_data
from core.generator import generate_captions, generate_videos_info, load_json
from core.renderer import render_videos
from core.wwiser_runner import run_wwiser
from core.config import ConfigManager

ctk.set_appearance_mode("Dark")

# Premium Theme Constants
BG_COLOR = "#0D0D12"       # Deep dark background
CARD_COLOR = "#1A1A24"     # Slightly lighter card background
ACCENT_COLOR = "#00E5FF"   # Cyan Wuthering Waves accent
ACCENT_HOVER = "#00B8CC"
TEXT_COLOR = "#FFFFFF"
SUBTEXT_COLOR = "#9E9E9E"

FONT_MAIN = ("Segoe UI", 13)
FONT_HEADER = ("Segoe UI", 26, "bold")
FONT_SUBHEADER = ("Segoe UI", 16, "bold")
FONT_CONSOLE = ("Consolas", 12)

class App(ctk.CTk):
    def __init__(self):
        super().__init__()

        self.title("WuWa Cutscene Exporter - Premium Edition")
        self.geometry("1050x750")
        self.configure(fg_color=BG_COLOR)
        
        self.config_mgr = ConfigManager()
        
        # Grid layout for sidebar and main content
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)
        
        # Sidebar
        self.sidebar_frame = ctk.CTkFrame(self, width=220, corner_radius=0, fg_color=CARD_COLOR)
        self.sidebar_frame.grid(row=0, column=0, rowspan=2, sticky="nsew")
        self.sidebar_frame.grid_rowconfigure(5, weight=1) # Spacer
        
        self.logo_label = ctk.CTkLabel(self.sidebar_frame, text="Wuthering\nWaves\nExporter", font=("Segoe UI", 24, "bold"), text_color=ACCENT_COLOR)
        self.logo_label.grid(row=0, column=0, padx=20, pady=(30, 40))
        
        self.btn_nav_render = ctk.CTkButton(self.sidebar_frame, text="🎬 Renderer", font=FONT_SUBHEADER, fg_color="transparent", text_color=TEXT_COLOR, hover_color=BG_COLOR, anchor="w", command=self.show_render_tab)
        self.btn_nav_render.grid(row=1, column=0, padx=10, pady=5, sticky="ew")
        
        self.btn_nav_wwiser = ctk.CTkButton(self.sidebar_frame, text="🎙️ Wwiser Audio", font=FONT_SUBHEADER, fg_color="transparent", text_color=TEXT_COLOR, hover_color=BG_COLOR, anchor="w", command=self.show_wwiser_tab)
        self.btn_nav_wwiser.grid(row=2, column=0, padx=10, pady=5, sticky="ew")
        
        self.btn_nav_settings = ctk.CTkButton(self.sidebar_frame, text="⚙️ Settings", font=FONT_SUBHEADER, fg_color="transparent", text_color=TEXT_COLOR, hover_color=BG_COLOR, anchor="w", command=self.show_settings_tab)
        self.btn_nav_settings.grid(row=3, column=0, padx=10, pady=5, sticky="ew")
        
        # Version Label
        self.version_label = ctk.CTkLabel(self.sidebar_frame, text="v2.1 Premium", font=("Segoe UI", 11), text_color=SUBTEXT_COLOR)
        self.version_label.grid(row=6, column=0, padx=20, pady=20, sticky="s")
        
        # Main Content Area
        self.main_frame = ctk.CTkFrame(self, fg_color=BG_COLOR, corner_radius=0)
        self.main_frame.grid(row=0, column=1, sticky="nsew", padx=30, pady=20)
        self.main_frame.grid_rowconfigure(0, weight=1)
        self.main_frame.grid_columnconfigure(0, weight=1)
        
        self.frames = {}
        self.frames["render"] = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        self.frames["wwiser"] = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        self.frames["settings"] = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        
        for f in self.frames.values():
            f.grid(row=0, column=0, sticky="nsew")
        
        self._build_render_tab()
        self._build_wwiser_tab()
        self._build_settings_tab()
        
        # Bottom Console Area
        self.console_frame = ctk.CTkFrame(self, fg_color=CARD_COLOR, corner_radius=15)
        self.console_frame.grid(row=1, column=1, sticky="ew", padx=30, pady=(0, 20))
        
        self.progress_bar = ctk.CTkProgressBar(self.console_frame, progress_color=ACCENT_COLOR, fg_color=BG_COLOR, height=8)
        self.progress_bar.pack(padx=20, pady=(15, 5), fill="x")
        self.progress_bar.set(0)
        
        # Terminal-style Log Box
        self.log_box = ctk.CTkTextbox(self.console_frame, height=140, fg_color="#08080B", text_color="#00FF41", font=FONT_CONSOLE, corner_radius=8, border_width=1, border_color="#1E1E28")
        self.log_box.pack(padx=20, pady=(5, 15), fill="both", expand=True)
        self.log_box.bind("<Key>", self._prevent_typing)
        
        self.show_render_tab()
        
        self.log("System Initialized. Welcome to WuWa Cutscene Exporter Premium.")
        self.log("Check the Settings tab to ensure your tools are downloaded.")

    def show_render_tab(self):
        self.frames["render"].tkraise()
        self._set_active_nav(self.btn_nav_render)
        
    def show_wwiser_tab(self):
        self.frames["wwiser"].tkraise()
        self._set_active_nav(self.btn_nav_wwiser)
        
    def show_settings_tab(self):
        self.frames["settings"].tkraise()
        self._set_active_nav(self.btn_nav_settings)
        
    def _set_active_nav(self, active_btn):
        for btn in [self.btn_nav_render, self.btn_nav_wwiser, self.btn_nav_settings]:
            if btn == active_btn:
                btn.configure(fg_color=BG_COLOR, text_color=ACCENT_COLOR)
            else:
                btn.configure(fg_color="transparent", text_color=TEXT_COLOR)

    def _prevent_typing(self, event):
        if event.state & 0x0004 or event.state & 0x0008:
            if event.keysym.lower() in ('v', 'x'): return "break"
            if event.keysym.lower() == 'c': return None # Allow copy
            return None
        if event.keysym in ('Up', 'Down', 'Left', 'Right', 'Prior', 'Next', 'Home', 'End'):
            return None
        return "break"

    def _build_path_selector(self, parent, label_text, string_var, config_key, placeholder=""):
        frame = ctk.CTkFrame(parent, fg_color=CARD_COLOR, corner_radius=10)
        frame.pack(fill="x", pady=8)
        
        ctk.CTkLabel(frame, text=label_text, width=160, anchor="w", font=("Segoe UI", 13, "bold"), text_color=TEXT_COLOR).pack(side="left", padx=15, pady=10)
        
        entry = ctk.CTkEntry(frame, textvariable=string_var, placeholder_text=placeholder, fg_color=BG_COLOR, border_width=0, height=32)
        entry.pack(side="left", padx=10, fill="x", expand=True)
        string_var.trace_add("write", lambda *args: self.config_mgr.set(config_key, string_var.get()))
        
        btn = ctk.CTkButton(frame, text="Browse", width=70, height=32, fg_color="#2A2A35", hover_color="#3A3A45", command=lambda: self.browse_folder(string_var))
        btn.pack(side="right", padx=15)

    def browse_folder(self, var):
        folder = filedialog.askdirectory()
        if folder: var.set(folder)

    # --- Render Tab ---
    def _build_render_tab(self):
        ctk.CTkLabel(self.frames["render"], text="Video Renderer", font=FONT_HEADER, text_color=TEXT_COLOR).pack(anchor="w", pady=(0, 20))
        
        self.movies_path_var = ctk.StringVar(value=self.config_mgr.get("movies_path"))
        self._build_path_selector(self.frames["render"], "Movies Folder:", self.movies_path_var, "movies_path", "Path to Client/Content/Aki/Movies")
        
        # Options Card
        opt_card = ctk.CTkFrame(self.frames["render"], fg_color=CARD_COLOR, corner_radius=10)
        opt_card.pack(fill="x", pady=8)
        
        # Row 1: Character & Locale
        row1 = ctk.CTkFrame(opt_card, fg_color="transparent")
        row1.pack(fill="x", padx=15, pady=10)
        
        self.char_var = ctk.StringVar(value=self.config_mgr.get("char_selection") or "Both")
        ctk.CTkLabel(row1, text="Character:", font=FONT_MAIN, width=80, anchor="w").pack(side="left")
        def save_char(): self.config_mgr.set("char_selection", self.char_var.get())
        ctk.CTkSegmentedButton(row1, variable=self.char_var, values=["Girl", "Boy", "Both"], command=lambda v: save_char(), selected_color=ACCENT_COLOR, selected_hover_color=ACCENT_HOVER).pack(side="left", padx=10)
        
        ctk.CTkLabel(row1, text="Voice Locale:", font=FONT_MAIN, width=80, anchor="w").pack(side="left", padx=(30,0))
        self.locale_var = ctk.StringVar(value=self.config_mgr.get("locale_selection"))
        loc_entry = ctk.CTkEntry(row1, textvariable=self.locale_var, width=60, fg_color=BG_COLOR, border_width=0, placeholder_text="ja")
        loc_entry.pack(side="left", padx=10)
        self.locale_var.trace_add("write", lambda *args: self.config_mgr.set("locale_selection", self.locale_var.get()))
        
        # Row 2: Subtitle & BGM
        row2 = ctk.CTkFrame(opt_card, fg_color="transparent")
        row2.pack(fill="x", padx=15, pady=(0, 10))
        
        self.subtitle_mode_var = ctk.StringVar(value=self.config_mgr.get("subtitle_mode") or "Soft-sub")
        ctk.CTkLabel(row2, text="Subtitles:", font=FONT_MAIN, width=80, anchor="w").pack(side="left")
        sub_menu = ctk.CTkOptionMenu(row2, variable=self.subtitle_mode_var, values=["None", "Soft-sub", "Hard-sub"], fg_color=BG_COLOR, button_color="#2A2A35", button_hover_color="#3A3A45")
        sub_menu.pack(side="left", padx=10)
        self.subtitle_mode_var.trace_add("write", lambda *args: self.config_mgr.set("subtitle_mode", self.subtitle_mode_var.get()))
        
        self.custom_bgm_var = ctk.StringVar(value="")
        ctk.CTkLabel(row2, text="Custom BGM:", font=FONT_MAIN, width=90, anchor="w").pack(side="left", padx=(20,0))
        bgm_entry = ctk.CTkEntry(row2, textvariable=self.custom_bgm_var, width=150, fg_color=BG_COLOR, border_width=0, placeholder_text="Select audio...")
        bgm_entry.pack(side="left", fill="x", expand=True, padx=10)
        
        def select_bgm():
            f = filedialog.askopenfilename(title="Select Custom BGM", filetypes=[("Audio Files", "*.txtp *.wav *.mp3 *.wem")])
            if f: self.custom_bgm_var.set(f)
        ctk.CTkButton(row2, text="📁", width=32, fg_color="#2A2A35", hover_color="#3A3A45", command=select_bgm).pack(side="left")
        
        # List Card
        list_card = ctk.CTkFrame(self.frames["render"], fg_color=CARD_COLOR, corner_radius=10)
        list_card.pack(fill="both", expand=True, pady=8)
        
        top_list = ctk.CTkFrame(list_card, fg_color="transparent")
        top_list.pack(fill="x", padx=15, pady=10)
        self.btn_scan = ctk.CTkButton(top_list, text="🔍 Scan Cutscenes", fg_color="#2A2A35", hover_color="#3A3A45", font=("Segoe UI", 13, "bold"), command=self.start_scan)
        self.btn_scan.pack(side="left")
        
        self.btn_select_all = ctk.CTkButton(top_list, text="Select All", width=70, fg_color="transparent", text_color=ACCENT_COLOR, hover_color=BG_COLOR, command=self.select_all_clips)
        self.btn_select_all.pack(side="right")
        self.btn_deselect_all = ctk.CTkButton(top_list, text="Deselect All", width=80, fg_color="transparent", text_color=SUBTEXT_COLOR, hover_color=BG_COLOR, command=self.deselect_all_clips)
        self.btn_deselect_all.pack(side="right", padx=5)
        
        self.scroll_frame = ctk.CTkScrollableFrame(list_card, fg_color=BG_COLOR, corner_radius=8)
        self.scroll_frame.pack(fill="both", expand=True, padx=15, pady=(0, 15))
        self.clip_checkboxes = {}
        
        self.btn_run_pipeline = ctk.CTkButton(self.frames["render"], text="🚀 Start Video Rendering", height=45, fg_color=ACCENT_COLOR, text_color="#000000", hover_color=ACCENT_HOVER, font=("Segoe UI", 15, "bold"), command=self.start_pipeline)
        self.btn_run_pipeline.pack(pady=(10, 0), fill="x")

    # --- Wwiser Tab ---
    def _build_wwiser_tab(self):
        ctk.CTkLabel(self.frames["wwiser"], text="Wwiser Audio Extractor", font=FONT_HEADER, text_color=TEXT_COLOR).pack(anchor="w", pady=(0, 20))
        
        self.wwise_path_var = ctk.StringVar(value=self.config_mgr.get("wwise_path"))
        self._build_path_selector(self.frames["wwiser"], "WwiseAudio_Generated:", self.wwise_path_var, "wwise_path", "Path to WwiseAudio_Generated folder")
        
        info_card = ctk.CTkFrame(self.frames["wwiser"], fg_color=CARD_COLOR, corner_radius=10)
        info_card.pack(fill="x", pady=10)
        
        info_text = (
            "Wwiser is required to parse .bnk audio banks from the game into playable .txtp formats.\n"
            "This process may take a few minutes depending on your CPU, but it utilizes parallel\n"
            "processing to be as fast as possible. Run this step whenever you update the game files."
        )
        ctk.CTkLabel(info_card, text=info_text, font=FONT_MAIN, text_color=SUBTEXT_COLOR, justify="left").pack(padx=20, pady=20, anchor="w")
        
        self.btn_run_wwiser = ctk.CTkButton(self.frames["wwiser"], text="⚡ Extract Audio Banks", height=45, fg_color=ACCENT_COLOR, text_color="#000000", hover_color=ACCENT_HOVER, font=("Segoe UI", 15, "bold"), command=self.start_wwiser)
        self.btn_run_wwiser.pack(pady=20, fill="x")

    # --- Settings Tab ---
    def _build_settings_tab(self):
        ctk.CTkLabel(self.frames["settings"], text="Settings & Tools", font=FONT_HEADER, text_color=TEXT_COLOR).pack(anchor="w", pady=(0, 20))
        
        opt_card = ctk.CTkFrame(self.frames["settings"], fg_color=CARD_COLOR, corner_radius=10)
        opt_card.pack(fill="x", pady=8)
        
        # Subtitle Language
        lang_frame = ctk.CTkFrame(opt_card, fg_color="transparent")
        lang_frame.pack(fill="x", padx=15, pady=20)
        
        ctk.CTkLabel(lang_frame, text="Database Language (MultiText):", font=FONT_MAIN).pack(side="left", padx=(0, 20))
        
        self.textmap_lang_var = ctk.StringVar(value=self.config_mgr.get("textmap_language") or "th")
        langs = ["th", "ja", "en", "zh-Hans", "zh-Hant", "ko", "de", "es", "fr", "id", "pt", "ru", "vi"]
        lang_menu = ctk.CTkOptionMenu(lang_frame, variable=self.textmap_lang_var, values=langs, fg_color=BG_COLOR, button_color="#2A2A35", button_hover_color="#3A3A45")
        lang_menu.pack(side="left")
        self.textmap_lang_var.trace_add("write", lambda *args: self.config_mgr.set("textmap_language", self.textmap_lang_var.get()))
        
        # Action Buttons
        actions_card = ctk.CTkFrame(self.frames["settings"], fg_color=CARD_COLOR, corner_radius=10)
        actions_card.pack(fill="x", pady=8)
        
        btn_update_json = ctk.CTkButton(actions_card, text="🔄 Force Update JSON Data", fg_color="#2A2A35", hover_color="#3A3A45", height=40, font=("Segoe UI", 14, "bold"), command=self.update_json)
        btn_update_json.pack(fill="x", padx=20, pady=(20, 10))
        
        btn_download_tools = ctk.CTkButton(actions_card, text="⬇️ Download Missing Tools", fg_color="#2A2A35", hover_color="#3A3A45", height=40, font=("Segoe UI", 14, "bold"), command=self.download_tools)
        btn_download_tools.pack(fill="x", padx=20, pady=(10, 20))

    # --- Core Logic Functions ---
    def log(self, message):
        def _append():
            self.log_box.insert("end", "> " + message + "\n")
            self.log_box.see("end")
        self.after(0, _append)
        
    def clear_log(self):
        def _clear():
            self.log_box.delete("1.0", "end")
        self.after(0, _clear)

    def update_progress(self, percent):
        self.after(0, lambda: self.progress_bar.set(percent))

    def download_tools(self):
        threading.Thread(target=self._download_tools_thread, daemon=True).start()
        
    def _download_tools_thread(self):
        self.clear_log()
        self.log("Checking and downloading tools...")
        self.progress_bar.set(0)
        setup_tools(self.log, self.update_progress)
        self.progress_bar.set(1.0)
        self.log("Tools are ready.")
        
    def update_json(self):
        threading.Thread(target=self._update_json_thread, daemon=True).start()
        
    def _update_json_thread(self):
        self.clear_log()
        self.log("Updating JSON database...")
        self.progress_bar.set(0)
        setup_json_data(self.log, force_update=True, textmap_lang=self.textmap_lang_var.get(), progress_callback=self.update_progress)
        self.progress_bar.set(1.0)
        self.log("JSON database updated.")

    def start_wwiser(self):
        wwise_path = self.wwise_path_var.get()
        if not wwise_path:
            self.log("ERR: Please select WwiseAudio_Generated folder first.")
            return
        
        self.btn_run_wwiser.configure(state="disabled")
        self.progress_bar.configure(mode="indeterminate")
        self.progress_bar.start()
        threading.Thread(target=self._run_wwiser_thread, args=(wwise_path,), daemon=True).start()

    def _run_wwiser_thread(self, wwise_path):
        self.clear_log()
        try:
            self.log("=== Wwiser Extraction ===")
            setup_tools(self.log, self.update_progress)
            txtp_dir = os.path.join(wwise_path, "txtp")
            os.makedirs(txtp_dir, exist_ok=True)
            run_wwiser(wwise_path, txtp_dir, self.log)
        except Exception as e:
            self.log(f"ERR: {e}")
        finally:
            self.after(0, lambda: self.progress_bar.stop())
            self.after(0, lambda: self.progress_bar.configure(mode="determinate"))
            self.after(0, lambda: self.progress_bar.set(1.0))
            self.after(0, lambda: self.btn_run_wwiser.configure(state="normal"))

    def start_scan(self):
        movies_path = self.movies_path_var.get()
        wwise_path = self.wwise_path_var.get()
        
        if not movies_path or not wwise_path:
            self.log("ERR: Please select both Movies and WwiseAudio_Generated folders.")
            return
            
        movies_found = False
        for root, dirs, files in os.walk(movies_path):
            if any(f.endswith('.mp4') for f in files):
                movies_found = True
                break
        if not movies_found:
            self.log("ERR: No .mp4 files found in Movies folder (or its subfolders)!")
            return
            
        txtp_dir = os.path.join(wwise_path, "txtp")
        txtp_found = False
        if os.path.exists(txtp_dir):
            for root, dirs, files in os.walk(txtp_dir):
                if any(f.endswith('.txtp') for f in files):
                    txtp_found = True
                    break
        if not txtp_found:
            self.log("ERR: No .txtp files found in WwiseAudio_Generated/txtp! Did you run Wwiser extraction?")
            return
            
        self.btn_scan.configure(state="disabled")
        threading.Thread(target=self._scan_thread, args=(movies_path, txtp_dir), daemon=True).start()

    def _scan_thread(self, movies_path, txtp_dir):
        self.clear_log()
        try:
            self.log("Scanning videos...")
            setup_tools(self.log, self.update_progress)
            textmap_lang = self.config_mgr.get("textmap_language") or "th"
            setup_json_data(self.log, textmap_lang=textmap_lang, progress_callback=self.update_progress)
            
            generate_videos_info(movies_path, txtp_dir, self.locale_var.get(), self.log)
            videos_info = load_json("videos_info.json")
            
            self.after(0, self._populate_list, videos_info)
            self.log(f"Scan complete! Found {len(videos_info)} cutscenes.")
        except Exception as e:
            self.log(f"ERR: {e}")
        finally:
            self.after(0, lambda: self.btn_scan.configure(state="normal"))

    def _populate_list(self, videos_info):
        for widget in self.scroll_frame.winfo_children():
            widget.destroy()
        self.clip_checkboxes.clear()
        
        for v in videos_info:
            name = f"{v['CgName']} ({v['GirlOrBoy']})"
            var = ctk.BooleanVar(value=True)
            chk = ctk.CTkCheckBox(self.scroll_frame, text=name, variable=var, fg_color=ACCENT_COLOR, hover_color=ACCENT_HOVER)
            chk.pack(anchor="w", pady=4, padx=10)
            self.clip_checkboxes[name] = {"var": var, "data": v}

    def select_all_clips(self):
        for data in self.clip_checkboxes.values():
            data["var"].set(True)

    def deselect_all_clips(self):
        for data in self.clip_checkboxes.values():
            data["var"].set(False)

    def start_pipeline(self):
        if not hasattr(self, 'clip_checkboxes') or not self.clip_checkboxes:
            self.log("ERR: Please scan and select cutscenes first!")
            return
            
        selected_videos = [data["data"] for data in self.clip_checkboxes.values() if data["var"].get()]
        if not selected_videos:
            self.log("ERR: No cutscenes selected!")
            return
            
        if self.char_var.get() != "Both":
            selected_videos = [v for v in selected_videos if v["GirlOrBoy"] == self.char_var.get()]
            
        if not selected_videos:
            self.log("ERR: No cutscenes match the selected character!")
            return
            
        self.btn_run_pipeline.configure(state="disabled")
        self.progress_bar.set(0)
        threading.Thread(target=self._run_pipeline_thread, args=(selected_videos,), daemon=True).start()

    def _run_pipeline_thread(self, selected_videos):
        self.clear_log()
        try:
            self.log(f"=== Starting Rendering Pipeline for {len(selected_videos)} videos ===")
            self.log("Step 1: Generating Subtitles...")
            generate_captions(self.log)
            
            self.log("Step 2: Rendering Final Videos...")
            
            missing_audio_list = []
            for v in selected_videos:
                if not v.get("Sound"):
                    missing_audio_list.append(f"{v['CgName']} ({v['GirlOrBoy']})")
                    
            if missing_audio_list:
                self.log(f"WARN: {len(missing_audio_list)} clips are missing audio!")
                for ma in missing_audio_list:
                    self.log(f" - Missing audio for: {ma}")
            else:
                self.log("INFO: All matched clips have audio.")
                
            render_videos(selected_videos, self.subtitle_mode_var.get(), self.custom_bgm_var.get(), self.log, self.update_progress)
            
            self.log("=== Pipeline Completed ===")
            self.log("Check the 'Videos' folder for your cutscenes.")
        except Exception as e:
            self.log(f"ERR: {e}")
        finally:
            self.after(0, lambda: self.btn_run_pipeline.configure(state="normal"))
