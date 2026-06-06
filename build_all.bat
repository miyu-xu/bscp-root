@echo off
setlocal enabledelayedexpansion

:: Single host build script:
:: binder-rpc (CMake) -> copy libs for Rust -> cargo virtmgr+vm+crosvm -> dist\windows
:: Usage: build_all.bat              (incremental build)
::        build_all.bat --clean      (clean rebuild from scratch)
:: Default Rust triple matches MinGW binder-rpc; override with: set RUST_TARGET=...

set "REPO_ROOT=%~dp0"
if "%REPO_ROOT:~-1%"=="\" set "REPO_ROOT=%REPO_ROOT:~0,-1%"
set "OUT_ROOT=%REPO_ROOT%\out"
set "CMAKE_BUILD_DIR=%OUT_ROOT%\build_windows"
set "GFXSTREAM_BUILD_DIR=%OUT_ROOT%\gfxstream_build_windows"
if not defined ENABLE_GFXSTREAM_ANGLE set "ENABLE_GFXSTREAM_ANGLE=0"
if not defined ANGLE_ROOT set "ANGLE_ROOT=%REPO_ROOT%\..\angle"
if not defined AEMU_COMMON_PATH if exist "%REPO_ROOT%\hardware\google\aemu" set "AEMU_COMMON_PATH=%REPO_ROOT%\hardware\google\aemu"
if not defined AEMU_COMMON_PATH set "AEMU_COMMON_PATH=%REPO_ROOT%\..\aemu"
if not defined FLATBUFFERS_PATH set "FLATBUFFERS_PATH=%REPO_ROOT%\external\flatbuffers"
if not defined ANGLE_RUNTIME_DIR set "ANGLE_RUNTIME_DIR="
if not defined GFXSTREAM_PATH set "GFXSTREAM_PATH="
if not defined RUSTUP_TOOLCHAIN set "RUSTUP_TOOLCHAIN=stable"
if not defined CARGO_CMD set "CARGO_CMD=rustup run %RUSTUP_TOOLCHAIN% cargo"
if not defined RUSTC_CMD set "RUSTC_CMD=rustup run %RUSTUP_TOOLCHAIN% rustc"
if not defined CROSVM_FEATURES set "CROSVM_FEATURES=whpx,composite-disk,android-sparse"

if not defined RUST_TARGET set "RUST_TARGET=x86_64-pc-windows-gnu"
set "TOTAL_STEPS=4"
set "RUST_STEP=3"
set "DIST_STEP=4"
if "%ENABLE_GFXSTREAM_ANGLE%"=="1" (
    set "TOTAL_STEPS=5"
    set "RUST_STEP=4"
    set "DIST_STEP=5"
)

:: --clean option
set "CLEAN_BUILD=0"
:parse_args
if "%~1"=="--clean" (
    set "CLEAN_BUILD=1"
    shift
    goto parse_args
)
if "%~1"=="-c" (
    set "CLEAN_BUILD=1"
    shift
    goto parse_args
)
if "%~1"=="--help" (
    echo Usage: %~nx0 [--clean]
    echo   --clean  Force clean rebuild (removes CMake cache and cargo target dir)
    exit /b 0
)

if "%CLEAN_BUILD%"=="1" (
    echo === Clean rebuild requested ===
    if exist "%CMAKE_BUILD_DIR%" (
        echo Removing CMake build dir: %CMAKE_BUILD_DIR%
        rmdir /s /q "%CMAKE_BUILD_DIR%"
    )
    if exist "%GFXSTREAM_BUILD_DIR%" (
        echo Removing gfxstream build dir: %GFXSTREAM_BUILD_DIR%
        rmdir /s /q "%GFXSTREAM_BUILD_DIR%"
    )
    if exist "%OUT_ROOT%\target" (
        echo Removing cargo target dir
        rmdir /s /q "%OUT_ROOT%\target"
    )
    if exist "%OUT_ROOT%\dist" (
        echo Removing dist dir
        rmdir /s /q "%OUT_ROOT%\dist"
    )
    echo Clean done.
    echo.
)

echo === Host build (Windows) ===
echo REPO_ROOT=%REPO_ROOT%
echo RUST_TARGET=%RUST_TARGET%

where rustup >nul 2>&1
if errorlevel 1 (
    echo Error: rustup not on PATH
    exit /b 1
)
call %CARGO_CMD% -V
if errorlevel 1 (
    echo Error: failed to run cargo via "%CARGO_CMD%"
    exit /b 1
)
call %RUSTC_CMD% -V
if errorlevel 1 (
    echo Error: failed to run rustc via "%RUSTC_CMD%"
    exit /b 1
)

echo.
echo [1/!TOTAL_STEPS!] binder-rpc ^(CMake MinGW build^)
cd /d "%REPO_ROOT%"

:: Auto-detect MinGW-w64 in common locations
if not defined MINGW_PATH set "MINGW_PATH="
if not defined MINGW_SUBDIR set "MINGW_SUBDIR=mingw64"
if defined MINGW_PATH if not exist "%MINGW_PATH%\bin\g++.exe" set "MINGW_PATH="
if not defined MINGW_PATH if exist "C:\workspace\%MINGW_SUBDIR%\bin\g++.exe" set "MINGW_PATH=C:\workspace\%MINGW_SUBDIR%"
if not defined MINGW_PATH if exist "C:\tools\%MINGW_SUBDIR%\bin\g++.exe" set "MINGW_PATH=C:\tools\%MINGW_SUBDIR%"
if not defined MINGW_PATH if exist "C:\msys64\%MINGW_SUBDIR%\bin\g++.exe" set "MINGW_PATH=C:\msys64\%MINGW_SUBDIR%"
if not defined MINGW_PATH if exist "C:\mingw-w64\%MINGW_SUBDIR%\bin\g++.exe" set "MINGW_PATH=C:\mingw-w64\%MINGW_SUBDIR%"

set "CMAKE_PATH=C:\Program Files\CMake\bin"
if not defined MINGW_PATH (
    echo Error: MinGW-w64 (g++.exe) not found. Checked:
    echo   C:\workspace\%MINGW_SUBDIR%
    echo   C:\tools\%MINGW_SUBDIR%
    echo   C:\msys64\%MINGW_SUBDIR%
    echo   C:\mingw-w64\%MINGW_SUBDIR%
    echo.
    echo Install MinGW-w64 or set MINGW_PATH to the correct location, e.g.:
    echo   set "MINGW_PATH=C:\your\mingw64"
    exit /b 1
) else (
    echo MinGW-w64 found at: %MINGW_PATH%
)
set "MINGW_BIN=%MINGW_PATH%\bin"
set "MINGW_CMAKE_PATH=%MINGW_PATH:\=/%"
if not exist "%CMAKE_PATH%\cmake.exe" (
    echo Error: CMake not found at %CMAKE_PATH%
    exit /b 1
)
if not exist "%OUT_ROOT%" mkdir "%OUT_ROOT%"
if not exist "%CMAKE_BUILD_DIR%" mkdir "%CMAKE_BUILD_DIR%"
cd /d "%CMAKE_BUILD_DIR%"
set "PATH=%MINGW_BIN%;%PATH%"
"%CMAKE_PATH%\cmake.exe" ^
    -G "MinGW Makefiles" ^
    -DCMAKE_C_COMPILER="%MINGW_CMAKE_PATH%/bin/gcc.exe" ^
    -DCMAKE_CXX_COMPILER="%MINGW_CMAKE_PATH%/bin/g++.exe" ^
    -DCMAKE_MAKE_PROGRAM="%MINGW_CMAKE_PATH%/bin/mingw32-make.exe" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH="%MINGW_CMAKE_PATH%" ^
    -DCMAKE_FIND_ROOT_PATH="%MINGW_CMAKE_PATH%" ^
    -DCMAKE_FIND_ROOT_PATH_MODE_PROGRAM=NEVER ^
    -DCMAKE_FIND_ROOT_PATH_MODE_LIBRARY=ONLY ^
    -DCMAKE_FIND_ROOT_PATH_MODE_INCLUDE=ONLY ^
    "%REPO_ROOT%"
if errorlevel 1 (
    echo Error: cmake configure failed
    exit /b 1
)
"%MINGW_BIN%\mingw32-make.exe" -j4
if errorlevel 1 (
    echo Error: mingw32-make failed
    exit /b 1
)
cd /d "%REPO_ROOT%"

echo.
echo [2/!TOTAL_STEPS!] Copy libbinder-rpc into binder rust/sys/libs for linking
set "SYS_LIBS=%REPO_ROOT%\frameworks\native\libs\binder\rust\sys\libs"
if not exist "%SYS_LIBS%" mkdir "%SYS_LIBS%"
set "BLIB=%CMAKE_BUILD_DIR%\lib"
set "BBIN=%CMAKE_BUILD_DIR%\bin"
:: MinGW often places the shared library DLL next to executables (bin\), import lib in lib\.
set "DLL_SRC="
if exist "%BBIN%\libbinder-rpc.dll" set "DLL_SRC=%BBIN%\libbinder-rpc.dll"
if not defined DLL_SRC if exist "%BLIB%\libbinder-rpc.dll" set "DLL_SRC=%BLIB%\libbinder-rpc.dll"
if not defined DLL_SRC (
    echo Error: libbinder-rpc.dll not found under build_windows\bin or build_windows\lib
    exit /b 1
)
copy /Y "%DLL_SRC%" "%SYS_LIBS%\libbinder-rpc.dll" >nul
echo Copied libbinder-rpc.dll from %DLL_SRC%
if exist "%BLIB%\libbinder-rpc.dll.a" (
    copy /Y "%BLIB%\libbinder-rpc.dll.a" "%SYS_LIBS%\" >nul
    echo Copied libbinder-rpc.dll.a
)

echo.
if "%ENABLE_GFXSTREAM_ANGLE%"=="1" (
    echo [3/!TOTAL_STEPS!] gfxstream backend
    set "GFXSTREAM_LIB_PATH="
    if defined GFXSTREAM_PATH if exist "%GFXSTREAM_PATH%\gfxstream_backend.dll" set "GFXSTREAM_LIB_PATH=%GFXSTREAM_PATH%"
    if not defined GFXSTREAM_LIB_PATH if defined GFXSTREAM_PATH if exist "%GFXSTREAM_PATH%\libgfxstream_backend.dll" set "GFXSTREAM_LIB_PATH=%GFXSTREAM_PATH%"
    if defined GFXSTREAM_LIB_PATH (
        echo Using existing gfxstream backend: %GFXSTREAM_LIB_PATH%
        set "GFXSTREAM_PATH=%GFXSTREAM_LIB_PATH%"
    ) else (
        if not exist "%ANGLE_ROOT%" (
            echo Error: ANGLE_ROOT not found: %ANGLE_ROOT%
            exit /b 1
        )
        if not exist "%AEMU_COMMON_PATH%" (
            echo Error: AEMU_COMMON_PATH not found: %AEMU_COMMON_PATH%
            exit /b 1
        )
        if not exist "%FLATBUFFERS_PATH%" (
            echo Error: FLATBUFFERS_PATH not found: %FLATBUFFERS_PATH%
            exit /b 1
        )
        if not exist "%GFXSTREAM_BUILD_DIR%" mkdir "%GFXSTREAM_BUILD_DIR%"
        "%CMAKE_PATH%\cmake.exe" ^
            -S "%REPO_ROOT%\hardware\google\gfxstream" ^
            -B "%GFXSTREAM_BUILD_DIR%" ^
            -G "MinGW Makefiles" ^
            -DCMAKE_C_COMPILER="%MINGW_CMAKE_PATH%/bin/gcc.exe" ^
            -DCMAKE_CXX_COMPILER="%MINGW_CMAKE_PATH%/bin/g++.exe" ^
            -DCMAKE_MAKE_PROGRAM="%MINGW_CMAKE_PATH%/bin/mingw32-make.exe" ^
            -DCMAKE_BUILD_TYPE=Release ^
            -DCMAKE_PREFIX_PATH="%MINGW_CMAKE_PATH%" ^
            -DCMAKE_FIND_ROOT_PATH="%MINGW_CMAKE_PATH%" ^
            -DCMAKE_FIND_ROOT_PATH_MODE_PROGRAM=NEVER ^
            -DCMAKE_FIND_ROOT_PATH_MODE_LIBRARY=ONLY ^
            -DCMAKE_FIND_ROOT_PATH_MODE_INCLUDE=ONLY ^
            -DANGLE_PATH="%ANGLE_ROOT%" ^
            -DAEMU_COMMON_PATH="%AEMU_COMMON_PATH%" ^
            -DFLATBUFFERS_PATH="%FLATBUFFERS_PATH%"
        if errorlevel 1 (
            echo Error: gfxstream cmake configure failed. Set GFXSTREAM_PATH to a prebuilt backend or provide AEMU_COMMON_PATH and FLATBUFFERS_PATH.
            exit /b 1
        )
        "%MINGW_BIN%\mingw32-make.exe" -C "%GFXSTREAM_BUILD_DIR%" gfxstream_backend -j4
        if errorlevel 1 (
            echo Error: gfxstream_backend build failed
            exit /b 1
        )
        set "GFXSTREAM_PATH=%GFXSTREAM_BUILD_DIR%"
    )
)

echo [!RUST_STEP!/!TOTAL_STEPS!] Rust ^(virtmgr + vm + crosvm^)
set "PATH=%CMAKE_BUILD_DIR%\bin;%CMAKE_BUILD_DIR%\lib;%PATH%"
cd /d "%REPO_ROOT%"
set "CARGO_TARGET_DIR=%OUT_ROOT%\target"
if "%ENABLE_GFXSTREAM_ANGLE%"=="1" (
    if not exist "%GFXSTREAM_PATH%\gfxstream_backend.dll" if not exist "%GFXSTREAM_PATH%\libgfxstream_backend.dll" (
        echo Error: gfxstream_backend.dll not found under %GFXSTREAM_PATH%
        exit /b 1
    )
    echo ,%CROSVM_FEATURES%, | findstr /C:",gfxstream," >nul
    if errorlevel 1 set "CROSVM_FEATURES=%CROSVM_FEATURES%,gfxstream"
)
echo [cargo] virtmgr --release --target %RUST_TARGET%
call %CARGO_CMD% build --manifest-path "%REPO_ROOT%\packages\modules\Virtualization\android\virtmgr\Cargo.toml" --release --target "%RUST_TARGET%"
if errorlevel 1 (
    echo Error: virtmgr cargo build failed ^(target=%RUST_TARGET%^) >&2
    echo For MinGW toolchain issues, ensure gcc/g++ are on PATH and the 'binder' crate can compile. >&2
    exit /b 1
)
echo [cargo] vm --release --target %RUST_TARGET%
call %CARGO_CMD% build --manifest-path "%REPO_ROOT%\packages\modules\Virtualization\android\vm\Cargo.toml" --release --target "%RUST_TARGET%"
if errorlevel 1 (
    echo Error: vm cargo build failed ^(target=%RUST_TARGET%^) >&2
    exit /b 1
)
echo [cargo] crosvm --release -p crosvm --target %RUST_TARGET% --features %CROSVM_FEATURES%
cd /d "%REPO_ROOT%\external\crosvm"
call %CARGO_CMD% build --release -p crosvm --target "%RUST_TARGET%" --features "%CROSVM_FEATURES%"
if errorlevel 1 (
    echo Error: crosvm cargo build failed ^(target=%RUST_TARGET%, features=%CROSVM_FEATURES%^) >&2
    echo Check that Rust nightly toolchain is installed for the %RUST_TARGET% target. >&2
    exit /b 1
)
cd /d "%REPO_ROOT%"

echo.
echo [!DIST_STEP!/!TOTAL_STEPS!] Collect artifacts into dist\windows
set "DIST=%OUT_ROOT%\dist\windows"
set "DIST_BIN=%DIST%\bin"
set "DIST_LIB=%DIST%\lib"
set "README_FILE=%DIST%\README.txt"
if not exist "%DIST_BIN%" mkdir "%DIST_BIN%"
if not exist "%DIST_LIB%" mkdir "%DIST_LIB%"

set "TGT_OUT=%CARGO_TARGET_DIR%\%RUST_TARGET%\release"

copy /Y "%CMAKE_BUILD_DIR%\bin\*.exe" "%DIST_BIN%\" >nul 2>&1
copy /Y "%DLL_SRC%" "%DIST_LIB%\libbinder-rpc.dll" >nul
copy /Y "%DLL_SRC%" "%DIST_BIN%\libbinder-rpc.dll" >nul
if "%ENABLE_GFXSTREAM_ANGLE%"=="1" (
    copy /Y "%GFXSTREAM_PATH%\gfxstream_backend.dll" "%DIST_BIN%\" >nul 2>&1
    copy /Y "%GFXSTREAM_PATH%\libgfxstream_backend.dll" "%DIST_BIN%\" >nul 2>&1
)

if exist "%TGT_OUT%\virtmgr.exe" (
    copy /Y "%TGT_OUT%\virtmgr.exe" "%DIST_BIN%\" >nul
) else (
    echo Warning: virtmgr.exe not present for dist copy
)
if exist "%TGT_OUT%\vm.exe" (
    copy /Y "%TGT_OUT%\vm.exe" "%DIST_BIN%\" >nul
) else (
    echo Warning: vm.exe not present for dist copy
)
copy /Y "%TGT_OUT%\crosvm.exe" "%DIST_BIN%\" >nul
if errorlevel 1 (
    echo Error: copy crosvm.exe failed
    exit /b 1
)
for %%F in (libslirp-0.dll r8Brain.dll ucrtbased.dll vcruntime140d.dll) do (
    if exist "%TGT_OUT%\%%F" (
        copy /Y "%TGT_OUT%\%%F" "%DIST_BIN%\" >nul
    )
)
if "%ENABLE_GFXSTREAM_ANGLE%"=="1" (
    if not defined ANGLE_RUNTIME_DIR call :find_angle_runtime_dir
    if not defined ANGLE_RUNTIME_DIR (
        echo Error: ANGLE runtime libraries not found. Set ANGLE_RUNTIME_DIR or build ANGLE under %ANGLE_ROOT%\out.
        exit /b 1
    )
    if not exist "%ANGLE_RUNTIME_DIR%\libEGL.dll" (
        echo Error: %ANGLE_RUNTIME_DIR%\libEGL.dll not found
        exit /b 1
    )
    if not exist "%ANGLE_RUNTIME_DIR%\libGLESv2.dll" (
        echo Error: %ANGLE_RUNTIME_DIR%\libGLESv2.dll not found
        exit /b 1
    )
    if not exist "%DIST%\gfx\angle" mkdir "%DIST%\gfx\angle"
    copy /Y "%ANGLE_RUNTIME_DIR%\libEGL.dll" "%DIST%\gfx\angle\" >nul
    copy /Y "%ANGLE_RUNTIME_DIR%\libGLESv2.dll" "%DIST%\gfx\angle\" >nul
    if exist "%ANGLE_RUNTIME_DIR%\libGLESv1_CM.dll" (
        copy /Y "%ANGLE_RUNTIME_DIR%\libGLESv1_CM.dll" "%DIST%\gfx\angle\" >nul
    )
    > "%DIST_BIN%\crosvm-angle.bat" echo @echo off
    >>"%DIST_BIN%\crosvm-angle.bat" echo setlocal
    >>"%DIST_BIN%\crosvm-angle.bat" echo set "SCRIPT_DIR=%%~dp0"
    >>"%DIST_BIN%\crosvm-angle.bat" echo set "DIST_ROOT=%%SCRIPT_DIR%%.."
    >>"%DIST_BIN%\crosvm-angle.bat" echo set "GFXSTREAM_ANGLE_ROOT=%%DIST_ROOT%%\gfx\angle"
    >>"%DIST_BIN%\crosvm-angle.bat" echo set "PATH=%%DIST_ROOT%%\lib;%%DIST_ROOT%%\gfx\angle;%%PATH%%"
    >>"%DIST_BIN%\crosvm-angle.bat" echo "%%SCRIPT_DIR%%crosvm.exe" %%*
)

> "%README_FILE%" echo build_all: OK
>>"%README_FILE%" echo RUST_TARGET=%RUST_TARGET%
>>"%README_FILE%" echo.
>>"%README_FILE%" echo bin: virtmgr.exe vm.exe crosvm.exe
>>"%README_FILE%" echo binder-dll: bin\libbinder-rpc.dll and lib\libbinder-rpc.dll
>>"%README_FILE%" echo runtime-dlls: libslirp-0.dll r8Brain.dll [ucrtbased.dll vcruntime140d.dll when present]
>>"%README_FILE%" echo lib: libbinder-rpc.dll
if "%ENABLE_GFXSTREAM_ANGLE%"=="1" (
    >>"%README_FILE%" echo gfx-angle wrapper: bin\crosvm-angle.bat
)

echo.
echo [verify] Checking build artifacts...
set "VERIFY_FAIL=0"

if not exist "%DIST_BIN%\virtmgr.exe" (
    echo   [FAIL] virtmgr.exe not found
    set VERIFY_FAIL=1
) else (
    echo   [OK]   virtmgr.exe
)
if not exist "%DIST_BIN%\vm.exe" (
    echo   [FAIL] vm.exe not found
    set VERIFY_FAIL=1
) else (
    echo   [OK]   vm.exe
)
if not exist "%DIST_BIN%\crosvm.exe" (
    echo   [FAIL] crosvm.exe not found
    set VERIFY_FAIL=1
) else (
    echo   [OK]   crosvm.exe
)
if not exist "%DIST_LIB%\libbinder-rpc.dll" (
    echo   [FAIL] libbinder-rpc.dll not found
    set VERIFY_FAIL=1
) else (
    echo   [OK]   libbinder-rpc.dll
)

if "%VERIFY_FAIL%"=="1" (
    echo   [FAIL] One or more artifacts are missing.
    exit /b 1
) else (
    echo   [PASS] All artifacts present
)

cd /d "%REPO_ROOT%"
echo.
echo Build completed successfully.
echo Artifacts: %DIST%
exit /b 0

:find_angle_runtime_dir
if not exist "%ANGLE_ROOT%\out" exit /b 0
for /r "%ANGLE_ROOT%\out" %%F in (libEGL.dll) do (
    set "ANGLE_RUNTIME_DIR=%%~dpF"
    exit /b 0
)
exit /b 0
