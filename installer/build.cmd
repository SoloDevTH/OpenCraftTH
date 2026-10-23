@echo off
rem Double-click: build every app and the one-file OpenCraft Studio installer (installer\dist).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build.ps1" -Build %*
pause
