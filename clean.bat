@echo off
chcp 65001 >nul
echo ========================================================
echo WuWa Cutscene Exporter - Project Reset Script
echo ========================================================
echo.
echo This script will remove all downloaded tools, generated
echo videos, subtitles, and cached JSON data to return the 
echo project to its clean starting state.
echo.
echo [WARNING] All rendered videos in the "Videos" folder 
echo and downloaded tools in the "tools" folder will be DELETED!
echo.
echo Press Ctrl+C to cancel, or any other key to proceed.
pause

echo.
echo Cleaning up folders...

if exist "Captions" (
    echo Deleting Captions...
    rmdir /s /q "Captions"
)

if exist "Videos" (
    echo Deleting Videos...
    rmdir /s /q "Videos"
)

if exist "Sounds" (
    echo Deleting Sounds...
    rmdir /s /q "Sounds"
)

if exist "tools" (
    echo Deleting tools...
    rmdir /s /q "tools"
)

if exist "data" (
    echo Deleting data...
    rmdir /s /q "data"
)

if exist "config.json" (
    echo Deleting config.json...
    del /q "config.json"
)

echo Cleaning up PyInstaller build files...
if exist "build" (
    echo Deleting build folder...
    rmdir /s /q "build"
)

if exist "dist" (
    echo Deleting dist folder...
    rmdir /s /q "dist"
)

if exist "*.spec" (
    echo Deleting .spec files...
    del /q "*.spec"
)

echo Cleaning up __pycache__ folders...
for /d /r . %%d in (__pycache__) do @if exist "%%d" rd /s /q "%%d"

echo.
echo Cleanup complete! Project has been reset to its initial state.
pause
