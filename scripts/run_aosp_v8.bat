@echo off
set "CROSVM_DIR=C:\workspace\bscp\bscp\out\dist\windows\bin"
set "ANGLE_DIR=C:\workspace\bscp\angle\out\Release-GfxAngle-Clang"
set "PATH=%CROSVM_DIR%;%ANGLE_DIR%;%PATH%"
set "KERNEL=C:\workspace\bscp\bscp\out\dist\img\android_kernel_aosp66"
set "DISK=C:\workspace\bscp\bscp\out\dist\img\aggregate_aosp.img"
set "INITRD=C:\workspace\bscp\bscp\out\dist\img\ramdisk_noavb.gz"
set "LOG_DIR=C:\workspace\bscp\bscp\out\dist\logs\android"
mkdir "%LOG_DIR%" 2>nul

echo ================================================
echo  AOSP v8 Boot Test (AOSP 6.6.30 kernel + SELinux off)
echo  Kernel: AOSP 6.6.30 (property_service class)
echo  Disk: aggregate_aosp.img (super_v8, ext4)
echo  SELinux: kernel disabled + userspace disabled
echo ================================================

"%CROSVM_DIR%\crosvm.exe" --log-level info run-mp --disable-sandbox --cid 100 --mem 4096 --cpus 4 --no-balloon --no-usb ^
  --serial "type=file,path=%LOG_DIR%\serial_aosp_v8.txt,hardware=serial,num=1,earlycon=true" ^
  --serial "type=file,path=%LOG_DIR%\hvc_aosp_v8.txt,hardware=virtio-console,num=1,console=true" ^
  --block "path=%DISK%,ro=false,lock=false,sparse=false" ^
  --initrd "%INITRD%" ^
  --params "console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init androidboot.hardware=cutf_cvm selinux=0 androidboot.selinux=disabled androidboot.slot_suffix=_a" ^
  --gpu "backend=gfxstream,width=1280,height=720,angle=true,vulkan=true,wsi=vk" ^
  "%KERNEL%" 2>%LOG_DIR%\stderr_aosp_v8.txt
echo Exit: %ERRORLEVEL%
pause
