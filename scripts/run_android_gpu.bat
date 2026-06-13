@echo off
setlocal enabledelayedexpansion
set "ANGLE_DIR=C:\workspace\bscp\angle\out\Release-GfxAngle-Clang"
set "CROSVM_DIR=C:\workspace\bscp\bscp\out\dist\windows\bin"
set "MINGW_DIR=C:\workspace\mingw64\bin"
set "GFX_DIR=C:\workspace\bscp\bscp\out\dist\windows\gfx\angle"
set "PATH=%ANGLE_DIR%;%CROSVM_DIR%;%MINGW_DIR%;%GFX_DIR%;%PATH%"
set "KERNEL=C:\workspace\bscp\bscp\out\dist\img\android_kernel"
set "DISK=C:\workspace\bscp\bscp\out\dist\img\aggregate.img"
set "INITRD=C:\workspace\bscp\bscp\out\dist\img\ramdisk_noavb.gz"
set "LOG_DIR=C:\workspace\bscp\bscp\out\dist\logs\android"

echo ================================================
echo  Google Play PC Android - GPU Boot
echo ================================================
echo Kernel: %KERNEL%
echo Disk: %DISK%
echo Initrd: %INITRD%
echo GPU: gfxstream 1280x720
echo ================================================

"%CROSVM_DIR%\crosvm.exe" --log-level info run-mp --disable-sandbox --cid 100 --mem 4096 --cpus 4 --no-balloon --no-usb ^
  --serial "type=file,path=%LOG_DIR%\android_serial.txt,hardware=serial,num=1,earlycon=true" ^
  --serial "type=file,path=%LOG_DIR%\android_hvc.txt,hardware=virtio-console,num=1,console=true" ^
  --block "path=%DISK%,ro=false,lock=false,sparse=false" ^
  --initrd "%INITRD%" ^
  --params "console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init androidboot.hardware=cutf_cvm androidboot.selinux=permissive androidboot.slot_suffix=_a" ^
  --gpu "backend=gfxstream,width=1280,height=720,angle=true,vulkan=true,wsi=vk" ^
  "%KERNEL%" 2>%LOG_DIR%\android_stderr.txt 1>%LOG_DIR%\android_stdout.txt

echo.
echo Exit code: %ERRORLEVEL%
pause
