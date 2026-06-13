@echo off
set "CROSVM_DIR=C:\workspace\bscp\bscp\out\dist\windows\bin"
set "MINGW_DIR=C:\workspace\mingw64\bin"
set "AFX_DIR=C:\workspace\bscp\bscp\out\dist\windows\gfx\angle"
set "ANGLE_DIR=C:\workspace\bscp\angle\out\Release-GfxAngle-Clang"
set "PATH=%CROSVM_DIR%;%MINGW_DIR%;%AFX_DIR%;%ANGLE_DIR%;%PATH%"
set "KERNEL=C:\workspace\bscp\bscp\out\dist\img\android_kernel"
set "DISK=C:\workspace\bscp\bscp\out\dist\img\aggregate.img"
set "INITRD=C:\workspace\bscp\bscp\out\dist\img\ramdisk_noavb.gz"
set "LOG_DIR=C:\workspace\bscp\bscp\out\dist\logs\android"
mkdir "%LOG_DIR%" 2>nul

echo ================================================
echo  Headless Boot Test (No GPU) - Verify baseline
echo ================================================

"%CROSVM_DIR%\crosvm.exe" --log-level info run-mp --disable-sandbox --cid 100 --mem 4096 --cpus 4 --no-balloon --no-usb ^
  --serial "type=file,path=%LOG_DIR%\serial_headless.txt,hardware=serial,num=1,earlycon=true" ^
  --serial "type=file,path=%LOG_DIR%\hvc_headless.txt,hardware=virtio-console,num=1,console=true" ^
  --block "path=%DISK%,ro=false,lock=false,sparse=false" ^
  --initrd "%INITRD%" ^
  --params "console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init androidboot.hardware=cutf_cvm androidboot.selinux=permissive androidboot.slot_suffix=_a" ^
  "%KERNEL%" 2>%LOG_DIR%\stderr_headless.txt

echo Exit: %ERRORLEVEL%
pause
