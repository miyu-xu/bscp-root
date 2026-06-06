@echo off
setlocal enabledelayedexpansion
set "ANGLE_DIR=C:\workspace\bscp\angle\out\Release-GfxAngle-Clang"
set "CROSVM_DIR=C:\workspace\bscp\bscp\out\dist\windows\bin"
set "MINGW_DIR=C:\workspace\mingw64\bin"
set "PATH=%ANGLE_DIR%;%CROSVM_DIR%;%MINGW_DIR%;%PATH%"
set "DISK=C:\workspace\bscp\bscp\out\dist\img\test_image_autologin.bin"
set "LOG_DIR=C:\workspace\bscp\bscp\out\dist\logs\chromeos-fw"
mkdir "%LOG_DIR%" 2>nul

echo ================================================
echo  ChromeOS OVMF Firmware Boot - %date% %time%
echo ================================================
echo Disk: %DISK%
echo BIOS: OVMF_DEBUG.fd
echo GPUs: gfxstream 1024x768
echo CPUs: 2
echo ================================================

"%CROSVM_DIR%\crosvm.exe" --log-level info run-mp --disable-sandbox --cid 100 --mem 4096 --cpus 2 --no-balloon --no-usb --serial type=file,path=%LOG_DIR%\serial.txt,hardware=serial,num=1,earlycon=true --serial type=sink,hardware=serial,num=2 --serial type=file,path=%LOG_DIR%\hvc.txt,hardware=virtio-console,num=1,console=true --block path=%DISK%,ro=false,lock=false,sparse=false,bootindex=1 --gpu backend=gfxstream,width=1024,height=768,angle=true,vulkan=true,wsi=vk --bios C:\workspace\bscp\bscp\out\dist\firmware\OVMF_DEBUG.fd 2>%LOG_DIR%\stderr.txt 1>%LOG_DIR%\stdout.txt

echo.
echo Exit code: %ERRORLEVEL%
pause
