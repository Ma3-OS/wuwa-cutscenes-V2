@echo off
title Wuwa Cutscenes - Run Executable
echo Starting Wuwa Cutscenes App (Executable Mode)...
if exist "src-tauri\target\release\tauri-app.exe" (
    cd src-tauri\target\release
    start tauri-app.exe
) else if exist "src-tauri\target\debug\tauri-app.exe" (
    cd src-tauri\target\debug
    start tauri-app.exe
) else (
    echo Executable not found! Please run 'run_app.bat' or 'build_release.bat' first.
    pause
)
