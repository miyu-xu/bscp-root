@echo off
setlocal enabledelayedexpansion

:: HD feature build. This file exists only on the hd-feature branch.
set "REPO_ROOT=%~dp0"
if "%REPO_ROOT:~-1%"=="\" set "REPO_ROOT=%REPO_ROOT:~0,-1%"
if not defined RUST_TARGET set "RUST_TARGET=x86_64-pc-windows-gnu"
if not defined RUSTUP_TOOLCHAIN set "RUSTUP_TOOLCHAIN=1.94.0-x86_64-pc-windows-gnu"
if not defined CARGO_CMD set "CARGO_CMD=rustup run %RUSTUP_TOOLCHAIN% cargo"

if not exist "%REPO_ROOT%\hd\Cargo.toml" (
    echo Error: HD repository is not present. Sync the hd-feature manifest. >&2
    exit /b 1
)

call "%REPO_ROOT%\build_all.bat" %*
if errorlevel 1 exit /b 1

set "CARGO_TARGET_DIR=%REPO_ROOT%\hd\target"
call "%REPO_ROOT%\hd\build.bat"
if errorlevel 1 (
    echo Error: HD workspace build failed. >&2
    exit /b 1
)

set "HD_RELEASE=%CARGO_TARGET_DIR%\%RUST_TARGET%\release"
set "DIST=%REPO_ROOT%\out\dist\windows"
set "DIST_BIN=%DIST%\bin"
if not exist "%DIST_BIN%" mkdir "%DIST_BIN%"

for %%F in (hd.exe hdctl.exe hd-host.exe hd-worker.exe hd-device-sim.exe hd-adb-bridge.exe hd-casimir-adapter.exe hd-rootcanal-adapter.exe hd-frame-producer.exe hd-uwb-adapter.exe hd-modem-adapter.exe hd-network-adapter.exe hd-audio-adapter.exe hd-camera-adapter.exe) do (
    if not exist "%HD_RELEASE%\%%F" (
        echo Error: missing HD release artifact: %%F >&2
        exit /b 1
    )
    copy /Y "%HD_RELEASE%\%%F" "%DIST_BIN%\%%F" >nul
    if errorlevel 1 exit /b 1
)

if exist "%DIST_BIN%\ui" rmdir /S /Q "%DIST_BIN%\ui"
if not exist "%REPO_ROOT%\hd\web\dist\index.html" (
    echo Error: HD web bundle is missing. Build the web workspace first. >&2
    exit /b 1
)
xcopy /E /I /Y /Q "%REPO_ROOT%\hd\web\dist" "%DIST_BIN%\ui" >nul
if errorlevel 2 exit /b 1

set "WEBVIEW2_LOADER="
for /d %%D in ("%CARGO_TARGET_DIR%\%RUST_TARGET%\release\build\webview2-com-sys-*") do (
    if exist "%%~fD\out\x64\WebView2Loader.dll" set "WEBVIEW2_LOADER=%%~fD\out\x64\WebView2Loader.dll"
)
if not defined WEBVIEW2_LOADER (
    echo Error: WebView2Loader.dll was not produced for the HD UI. >&2
    exit /b 1
)
copy /Y "!WEBVIEW2_LOADER!" "%DIST_BIN%\WebView2Loader.dll" >nul
if errorlevel 1 exit /b 1

if not exist "%DIST%\docs\hd" mkdir "%DIST%\docs\hd"
copy /Y "%REPO_ROOT%\hd\README.md" "%DIST%\docs\hd\" >nul
if exist "%REPO_ROOT%\hd\README.zh-CN.md" copy /Y "%REPO_ROOT%\hd\README.zh-CN.md" "%DIST%\docs\hd\" >nul
copy /Y "%REPO_ROOT%\hd\LICENSE" "%DIST%\docs\hd\" >nul

if not defined MINGW_PATH set "MINGW_PATH=C:\msys64\mingw64"
call %CARGO_CMD% run --manifest-path "%REPO_ROOT%\hd\Cargo.toml" --target "%RUST_TARGET%" -p xtask -- pe-audit --bin-dir "%DIST_BIN%" --objdump "%MINGW_PATH%\bin\objdump.exe"
if errorlevel 1 (
    echo Error: HD PE dependency audit failed. >&2
    exit /b 1
)

echo HD feature artifacts: %DIST%
exit /b 0
