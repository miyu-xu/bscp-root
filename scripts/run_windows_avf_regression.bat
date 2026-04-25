@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0run_windows_avf_regression.ps1" %*
exit /b %ERRORLEVEL%
