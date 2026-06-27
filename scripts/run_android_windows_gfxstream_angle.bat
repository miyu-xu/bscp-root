@echo off
setlocal
powershell -ExecutionPolicy Bypass -File "%~dp0run_android_windows_gfxstream_angle.ps1" %*
