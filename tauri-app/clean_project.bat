@echo off
title Clean Project (Reduce Size for GitHub/Zip)
echo -----------------------------------
echo  Cleaning build files and node_modules...
echo  This will make the folder very small!
echo -----------------------------------
echo.

echo Removing node_modules...
if exist "node_modules" rmdir /s /q "node_modules"

echo Removing Tauri build target (src-tauri\target)...
if exist "src-tauri\target" rmdir /s /q "src-tauri\target"

echo Removing Frontend dist folder...
if exist "dist" rmdir /s /q "dist"

echo.
echo Clean complete! The folder is now tiny and ready for GitHub or Zipping.
echo Note: You will need to run 'npm install' or 'run_app.bat' to download dependencies again before you can run the app.
pause
