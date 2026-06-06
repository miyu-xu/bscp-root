@echo off
setlocal enabledelayedexpansion
set "ANGLE_DIR=C:\workspace\bscp\angle\out\Release-GfxAngle-Clang"
set "CROSVM_DIR=C:\workspace\bscp\bscp\out\dist\windows\bin"
set "MINGW_DIR=C:\workspace\mingw64\bin"
set "PATH=%ANGLE_DIR%;%CROSVM_DIR%;%MINGW_DIR%;%PATH%"
set "KERNEL=C:\workspace\bscp\bscp\out\dist\img\custom_kernel"
set "DISK=C:\workspace\bscp\bscp\out\dist\img\test_image_autologin.bin"
set "INITRD=C:\workspace\bscp\bscp\out\dist\img\initramfs.cpio.gz"
set "TPM_NVRAM_PATH=C:\workspace\bscp\bscp\out\dist\img\tpm_nvram.bin"
set "LOG_DIR=C:\workspace\bscp\bscp\out\dist\logs\chromeos"
mkdir "%LOG_DIR%" 2>nul

set BOOT=0
set MAX_BOOT=10
set RESTART_CODE=-536870788

:loop
set /a BOOT+=1
echo ================================================
echo  ChromeOS Boot #!BOOT! - %date% %time%
echo ================================================

"%CROSVM_DIR%\crosvm.exe" --log-level info run-mp --disable-sandbox --cid 100 --mem 4096 --cpus 2 --no-balloon --no-usb --serial type=file,path=%LOG_DIR%\serial.txt,hardware=serial,num=1,earlycon=true --serial type=sink,hardware=serial,num=2 --serial type=file,path=%LOG_DIR%\hvc.txt,hardware=virtio-console,num=1,console=true --block path=%DISK%,ro=false,lock=false,sparse=false --initrd %INITRD% --params "root=/dev/vda3 rw loglevel=7 console=ttyS0,115200n8 rootwait cros_debug" --gpu backend=gfxstream,width=1024,height=768,angle=true,vulkan=true,wsi=vk %KERNEL% 2>%LOG_DIR%\stderr.!BOOT!.txt 1>%LOG_DIR%\stdout.!BOOT!.txt

set EXIT_CODE=%ERRORLEVEL%
echo Exit code: !EXIT_CODE!

:: Guest reboot (ACPI S5 restart or clean shutdown) = restart crosvm
if !EXIT_CODE! equ %RESTART_CODE% (
    if !BOOT! lss %MAX_BOOT% (
        echo Guest reboot detected - restarting crosvm...
        timeout /t 2 >nul
        goto loop
    )
)

:: Any other exit = stop
if !BOOT! geq %MAX_BOOT% (
    echo Reached max boots (%MAX_BOOT%)
)
echo Done.
pause
