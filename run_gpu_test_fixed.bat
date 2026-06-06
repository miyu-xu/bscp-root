@echo off
setlocal enabledelayedexpansion
set "ANGLE_DIR=C:\workspace\bscp\angle\out\Release-GfxAngle-Clang"
set "CROSVM_DIR=C:\workspace\bscp\bscp\out\dist\windows\bin"
set "MINGW_DIR=C:\workspace\mingw64\bin"
set "PATH=%ANGLE_DIR%;%CROSVM_DIR%;%MINGW_DIR%;%PATH%"
set "KERNEL=C:\workspace\bscp\bscp\out\dist\img\custom_kernel"
set "DISK=C:\workspace\bscp\bscp\out\dist\img\test_image_autologin.bin"
set "LOG_DIR=C:\workspace\bscp\bscp\out\dist\logs\gpu_fixed"
mkdir "%LOG_DIR%" 2>nul

echo ================================================
echo  ChromeOS GPU Test (RW root) - %date% %time%
echo ================================================

"%CROSVM_DIR%\crosvm.exe" --log-level warn run-mp --disable-sandbox --cid 200 --mem 4096 --cpus 2 --no-balloon --no-usb --serial type=file,path=%LOG_DIR%\serial.txt,hardware=serial,num=1,earlycon=true --serial type=sink,hardware=serial,num=2 --serial type=file,path=%LOG_DIR%\hvc.txt,hardware=virtio-console,num=1,console=true --block path=%DISK%,ro=false,lock=false,sparse=false --params "root=/dev/vda3 rw loglevel=7 console=ttyS0,115200n8 noinitrd init=/sbin/init rootwait cros_debug" --gpu backend=gfxstream,width=1024,height=768,angle=true,vulkan=true,wsi=vk %KERNEL% 2>%LOG_DIR%\stderr.txt

echo Exit: %ERRORLEVEL%
