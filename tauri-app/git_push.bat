@echo off
title Push to GitHub
echo -----------------------------------
echo  Auto-Commit and Push to GitHub (Only tauri-app)
echo -----------------------------------
echo.

if not exist ".git" (
    echo Initializing new Git repository in tauri-app...
    git init
    git branch -M main
)

git status
echo.

set /p commit_msg="Enter commit message (or press Enter for 'Auto-update'): "
if "%commit_msg%"=="" set commit_msg=Auto-update

echo.
echo Adding changes...
git add .

echo Committing...
git commit -m "%commit_msg%"

echo Pushing to GitHub...
git push

echo.
echo Done!
pause
