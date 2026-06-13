@echo off
set "CROSVM_DIR=C:\workspace\bscp\bscp\out\dist\windows\bin"
set "ANGLE_DIR=C:\workspace\bscp\angle\out\Release-GfxAngle-Clang"
set "PATH=%CROSVM_DIR%;%ANGLE_DIR%;%PATH%"
set "KERNEL=C:\workspace\bscp\bscp\out\dist\img\android_kernel"
set "DISK1=C:\workspace\bscp\bscp\out\dist\img\aggregate_NEW.img"
set "DISK2=C:\workspace\bscp\bscp\out\dist\img\aosp_vendor_ext4.img"
set "INITRD=C:\workspace\bscp\bscp\out\dist\img\ramdisk_noavb.gz"
set "LOG_DIR=C:\workspace\bscp\bscp\out\dist\logs\android"
mkdir "%LOG_DIR%" 2>nul

echo ================================================
echo  GPU Fix Final
echo  Disk1: aggregate.img (GPP super + mod vendor)
echo  Disk2: aosp_vendor_ext4.img (AOSP CF vendor)
echo  Original fstab (vendor from super)
echo  init.common.rc: mount vdb + bind-mount
echo ================================================

"%CROSVM_DIR%\crosvm.exe" --log-level info run-mp --disable-sandbox --cid 100 --mem 4096 --cpus 4 --no-balloon --no-usb ^
  --serial "type=file,path=%LOG_DIR%\serial_final.txt,hardware=serial,num=1,earlycon=true" ^
  --serial "type=file,path=%LOG_DIR%\hvc_final.txt,hardware=virtio-console,num=1,console=true" ^
  --block "path=%DISK1%,ro=false,lock=false,sparse=false" ^
  --block "path=%DISK2%,ro=false,lock=false,sparse=false" ^
  --initrd "%INITRD%" ^
  --params "console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init androidboot.hardware=cutf_cvm androidboot.selinux=permissive androidboot.slot_suffix=_a" ^
  --gpu "backend=gfxstream,width=1280,height=720,angle=true,vulkan=true,wsi=vk" ^
  "%KERNEL%" 2>%LOG_DIR%\stderr_final.txt
echo Exit: %ERRORLEVEL%
pause
