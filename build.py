import os
import subprocess

print("Installing requirements for building...")
subprocess.call(["pip", "install", "-r", "requirements.txt"])

import sys

print("Building executable with PyInstaller...")
cmd = [
    sys.executable, "-m", "PyInstaller",
    "--noconfirm",
    "--onefile",
    "--windowed",
    "--name", "WuWa_Cutscene_Exporter",
    "main.py"
]
subprocess.call(cmd)

print("Build complete! Check the 'dist' folder for WuWa_Cutscene_Exporter.exe")
