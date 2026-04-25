@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0check_windows_avf_markers.ps1" %*
exit /b %ERRORLEVEL%
