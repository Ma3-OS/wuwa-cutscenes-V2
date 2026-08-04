@echo off
title Wuwa Cutscenes - Build Release
echo Building Wuwa Cutscenes App (Release Mode) for Distribution...
npm run tauri build
echo.
echo Build finished! 
echo You can find the .exe file in: src-tauri\target\release\
pause
