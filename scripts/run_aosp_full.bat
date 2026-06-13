@echo off
set "CROSVM_DIR=C:\workspace\bscp\bscp\out\dist\windows\bin"
set "ANGLE_DIR=C:\workspace\bscp\angle\out\Release-GfxAngle-Clang"
set "PATH=%CROSVM_DIR%;%ANGLE_DIR%;%PATH%"
set "KERNEL=C:\workspace\bscp\bscp\out\dist\img\aosp_kernel"
set "DISK=C:\workspace\bscp\bscp\out\dist\img\aggregate_aosp.img"
set "INITRD=C:\workspace\bscp\bscp\out\dist\img\ramdisk_aosp_v2.gz"
set "LOG_DIR=C:\workspace\bscp\bscp\out\dist\logs\android"
mkdir "%LOG_DIR%" 2>nul

echo ================================================
echo  AOSP Full Boot
echo  Kernel: AOSP 6.6.30 (EROFS support)
echo  Disk: aggregate_aosp.img (AOSP super + boot_a)
echo  Ramdisk: hybrid (AOSP init + GPP modules)
echo ================================================

"%CROSVM_DIR%\crosvm.exe" --log-level info run-mp --disable-sandbox --cid 100 --mem 4096 --cpus 4 --no-balloon --no-usb ^
  --serial "type=file,path=%LOG_DIR%\serial_aosp_full.txt,hardware=serial,num=1,earlycon=true" ^
  --serial "type=file,path=%LOG_DIR%\hvc_aosp_full.txt,hardware=virtio-console,num=1,console=true" ^
  --block "path=%DISK%,ro=false,lock=false,sparse=false" ^
  --initrd "%INITRD%" ^
  --params "console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init androidboot.hardware=cutf_cvm androidboot.selinux=permissive androidboot.slot_suffix=_a" ^
  --gpu "backend=gfxstream,width=1280,height=720,angle=true,vulkan=true,wsi=vk" ^
  "%KERNEL%" 2>%LOG_DIR%\stderr_aosp_full.txt
echo Exit: %ERRORLEVEL%
pause
