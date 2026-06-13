@echo off
set "CROSVM_DIR=C:\workspace\bscp\bscp\out\dist\windows\bin"
set "ANGLE_DIR=C:\workspace\bscp\angle\out\Release-GfxAngle-Clang"
set "PATH=%CROSVM_DIR%;%ANGLE_DIR%;%PATH%"
set "KERNEL=C:\workspace\bscp\bscp\out\dist\img\android_kernel"
set "DISK1=C:\workspace\bscp\bscp\out\dist\img\aggregate.img"
set "DISK2=C:\workspace\bscp\bscp\out\dist\img\aosp_vendor_ext4.img"
set "INITRD=C:\workspace\bscp\bscp\out\dist\img\ramdisk_vdb_vendor2.gz"
set "LOG_DIR=C:\workspace\bscp\bscp\out\dist\logs\android"
mkdir "%LOG_DIR%" 2>nul

echo ================================================
echo  AOSP Vendor via /dev/block/vdb (raw ext4)
echo  Disk 1: aggregate.img (system + product)
echo  Disk 2: aosp_vendor_ext4.img (AOSP CF vendor)
echo  Fstab: /dev/block/vdb -> /vendor
echo ================================================

"%CROSVM_DIR%\crosvm.exe" --log-level info run-mp --disable-sandbox --cid 100 --mem 4096 --cpus 4 --no-balloon --no-usb ^
  --serial "type=file,path=%LOG_DIR%\serial_vdb.txt,hardware=serial,num=1,earlycon=true" ^
  --serial "type=file,path=%LOG_DIR%\hvc_vdb.txt,hardware=virtio-console,num=1,console=true" ^
  --block "path=%DISK1%,ro=false,lock=false,sparse=false" ^
  --block "path=%DISK2%,ro=false,lock=false,sparse=false" ^
  --initrd "%INITRD%" ^
  --params "console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init androidboot.hardware=cutf_cvm androidboot.selinux=permissive androidboot.slot_suffix=_a" ^
  --gpu "backend=gfxstream,width=1280,height=720,angle=true,vulkan=true,wsi=vk" ^
  "%KERNEL%" 2>%LOG_DIR%\stderr_vdb.txt
echo Exit: %ERRORLEVEL%
pause
