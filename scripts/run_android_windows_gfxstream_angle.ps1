param(
    [string]$ArtifactDir = "",
    [string]$WorkDir = "",
    [string]$LogDir = "",
    [int]$Mem = 8192,
    [int]$Cpus = 4,
    [int]$Cid = 100,
    [int]$TimeoutSecs = 0,
    [ValidateSet("run", "run-mp")]
    [string]$RunMode = "run-mp",
    [switch]$RefreshImages,
    [switch]$NoRun,
    [switch]$DryRun,
    [switch]$UseSwiftShader,
    [switch]$ConservativeWhpx,
    [switch]$FullHvc
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
if (-not $WorkDir) { $WorkDir = Join-Path $RepoRoot "out\android-windows" }
if (-not $LogDir) { $LogDir = Join-Path $RepoRoot "out\dist\logs\android-windows" }

function Find-DefaultArtifactDir {
    $root = "D:\bscp-vm-artifacts"
    if (-not (Test-Path $root)) {
        throw "Artifact root not found: $root"
    }

    $candidates = Get-ChildItem -Path $root -Directory -Filter "bscp-vm-artifacts-*" |
        ForEach-Object {
            Join-Path $_.FullName "products\android\vsoc_x86_64\direct-linux"
        } |
        Where-Object {
            Test-Path (Join-Path $_ "aggregate_android.img")
        } |
        ForEach-Object {
            Get-Item $_
        } |
        Sort-Object LastWriteTime -Descending

    if (-not $candidates) {
        throw "No Android direct-linux image set found under $root"
    }
    return $candidates[0].FullName
}

function Copy-IfNeeded {
    param(
        [Parameter(Mandatory = $true)][string]$Source,
        [Parameter(Mandatory = $true)][string]$Destination
    )

    if (-not (Test-Path $Source)) {
        throw "Missing source file: $Source"
    }

    $src = Get-Item -LiteralPath $Source
    $copy = $RefreshImages -or -not (Test-Path $Destination)
    if (-not $copy) {
        $dst = Get-Item -LiteralPath $Destination
        $copy = $dst.Length -ne $src.Length
    }

    if ($copy) {
        Write-Host "copy $Source -> $Destination"
        Copy-Item -LiteralPath $Source -Destination $Destination -Force
    } else {
        Write-Host "reuse $Destination"
    }
}

function Join-WindowsArgumentList {
    param([string[]]$Arguments)

    $quoted = foreach ($arg in $Arguments) {
        if ($arg -ne "" -and $arg -notmatch '[\s"]') {
            $arg
            continue
        }

        $builder = New-Object System.Text.StringBuilder
        [void]$builder.Append('"')
        $backslashes = 0
        foreach ($ch in $arg.ToCharArray()) {
            if ($ch -eq '\') {
                $backslashes++
                continue
            }
            if ($ch -eq '"') {
                [void]$builder.Append('\' * (($backslashes * 2) + 1))
                [void]$builder.Append('"')
                $backslashes = 0
                continue
            }
            if ($backslashes -gt 0) {
                [void]$builder.Append('\' * $backslashes)
                $backslashes = 0
            }
            [void]$builder.Append($ch)
        }
        if ($backslashes -gt 0) {
            [void]$builder.Append('\' * ($backslashes * 2))
        }
        [void]$builder.Append('"')
        $builder.ToString()
    }

    return ($quoted -join " ")
}

if (-not $ArtifactDir) {
    $ArtifactDir = Find-DefaultArtifactDir
}
$ArtifactDir = (Resolve-Path $ArtifactDir).Path

New-Item -ItemType Directory -Force -Path $WorkDir, $LogDir | Out-Null
$HvcDir = Join-Path $WorkDir "hvc"
New-Item -ItemType Directory -Force -Path $HvcDir | Out-Null

$files = @(
    "aggregate_android.img",
    "android_fstab.dt",
    "initrd_android.img",
    "kernel",
    "misc.img",
    "metadata.img",
    "factory_reset_protected.img",
    "android.dtb",
    "android_fstab_extra.cpio.lz4"
)

foreach ($name in $files) {
    if ($DryRun) {
        $src = Join-Path $ArtifactDir $name
        if (-not (Test-Path $src)) { throw "Missing source file: $src" }
        Write-Host "dry-run: would copy $src"
    } else {
        Copy-IfNeeded -Source (Join-Path $ArtifactDir $name) -Destination (Join-Path $WorkDir $name)
    }
}

$artifactHvc = Join-Path $ArtifactDir "hvc"
foreach ($port in @("keymaster", "gatekeeper", "bt", "gnss", "location", "confui", "uwb", "oemlock", "keymint", "nfc", "sensors", "mcu_control", "mcu_uart")) {
    $dst = Join-Path $HvcDir "$port.in"
    $src = Join-Path $artifactHvc "$port.in"
    if (Test-Path $src) {
        if ($DryRun) {
            Write-Host "dry-run: would copy $src"
        } else {
            Copy-IfNeeded -Source $src -Destination $dst
        }
    } elseif (-not (Test-Path $dst)) {
        if (-not $DryRun) {
            New-Item -ItemType File -Path $dst | Out-Null
        }
    }
}

$DistRoot = Join-Path $RepoRoot "out\dist\windows"
$BinDir = Join-Path $DistRoot "bin"
$Crosvm = Join-Path $BinDir "crosvm.exe"
if (-not (Test-Path $Crosvm)) {
    throw "crosvm.exe not found: $Crosvm"
}

$angleCandidates = @(
    (Join-Path $DistRoot "gfx\angle"),
    $BinDir,
    (Join-Path $RepoRoot "..\angle\out\Release-GfxAngle-Clang")
)
$AngleDir = $angleCandidates | Where-Object {
    (Test-Path (Join-Path $_ "libEGL.dll")) -and (Test-Path (Join-Path $_ "libGLESv2.dll"))
} | Select-Object -First 1
if ($AngleDir) {
    $env:GFXSTREAM_ANGLE_ROOT = $AngleDir
    $env:PATH = "$AngleDir;$BinDir;$env:PATH"
} else {
    $env:PATH = "$BinDir;$env:PATH"
}
$env:GFXSTREAM_PATH = $BinDir

if ($UseSwiftShader) {
    $swiftshaderIcd = Join-Path $BinDir "vk_swiftshader_icd.json"
    if (-not (Test-Path $swiftshaderIcd)) {
        throw "SwiftShader ICD not found: $swiftshaderIcd"
    }
    $env:VK_ICD_FILENAMES = $swiftshaderIcd
}

if ($ConservativeWhpx) {
    $Cpus = 1
    $env:CROSVM_WHPX_BLOCK_QUEUES = "1"
}

$Disk = Join-Path $WorkDir "aggregate_android.img"
$Fstab = Join-Path $WorkDir "android_fstab.dt"
$Initrd = Join-Path $WorkDir "initrd_android.img"
$Kernel = Join-Path $WorkDir "kernel"
$KernelParams = "console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init"
if ($ConservativeWhpx) {
    $KernelParams = "$KernelParams clearcpuid=297"
}

$GpuArg = "backend=gfxstream,displays=[[mode=windowed[1280,720],dpi=[320,320],refresh-rate=60]],context-types=gfxstream-vulkan:gfxstream-composer,angle=true,gles=false,vulkan=true,wsi=vk"

if (-not $NoRun -and -not $DryRun) {
    Get-ChildItem -LiteralPath $LogDir -File -Filter "*.txt" -ErrorAction SilentlyContinue |
        Remove-Item -Force -ErrorAction SilentlyContinue
}

$serialArgs = @(
    "--serial", "type=file,path=$(Join-Path $LogDir "serial.txt"),hardware=serial,num=1,earlycon=true",
    "--serial", "type=sink,hardware=serial,num=2",
    "--serial", "hardware=legacy-virtio-console,num=1,type=file,path=$(Join-Path $LogDir "hvc.txt"),console=true,pci-address=00:02.0"
)

if ($RunMode -ne "run-mp" -or $FullHvc) {
    $serialArgs += @(
        "--serial", "hardware=legacy-virtio-console,num=2,type=sink,pci-address=00:04.0",
        "--serial", "hardware=legacy-virtio-console,num=3,type=file,path=$(Join-Path $LogDir "logcat-hvc2.txt"),pci-address=00:05.0",
        "--serial", "hardware=legacy-virtio-console,num=4,type=file,path=$(Join-Path $LogDir "keymaster-hvc3.txt"),input=$(Join-Path $HvcDir "keymaster.in"),pci-address=00:06.0",
        "--serial", "hardware=legacy-virtio-console,num=5,type=file,path=$(Join-Path $LogDir "gatekeeper-hvc4.txt"),input=$(Join-Path $HvcDir "gatekeeper.in"),pci-address=00:07.0",
        "--serial", "hardware=legacy-virtio-console,num=6,type=sink,pci-address=00:08.0",
        "--serial", "hardware=legacy-virtio-console,num=7,type=sink,pci-address=00:09.0",
        "--serial", "hardware=legacy-virtio-console,num=8,type=sink,pci-address=00:0a.0",
        "--serial", "hardware=legacy-virtio-console,num=9,type=file,path=$(Join-Path $LogDir "confui-hvc8.txt"),input=$(Join-Path $HvcDir "confui.in"),pci-address=00:0b.0",
        "--serial", "hardware=legacy-virtio-console,num=10,type=file,path=$(Join-Path $LogDir "uwb-hvc9.txt"),input=$(Join-Path $HvcDir "uwb.in"),pci-address=00:0c.0",
        "--serial", "hardware=legacy-virtio-console,num=11,type=file,path=$(Join-Path $LogDir "oemlock-hvc10.txt"),input=$(Join-Path $HvcDir "oemlock.in"),pci-address=00:0d.0",
        "--serial", "hardware=legacy-virtio-console,num=12,type=file,path=$(Join-Path $LogDir "keymint-hvc11.txt"),input=$(Join-Path $HvcDir "keymint.in"),pci-address=00:0e.0",
        "--serial", "hardware=legacy-virtio-console,num=13,type=sink,pci-address=00:0f.0",
        "--serial", "hardware=legacy-virtio-console,num=14,type=file,path=$(Join-Path $LogDir "sensors-hvc13.txt"),input=$(Join-Path $HvcDir "sensors.in"),pci-address=00:10.0",
        "--serial", "hardware=legacy-virtio-console,num=15,type=file,path=$(Join-Path $LogDir "mcu-control-hvc14.txt"),input=$(Join-Path $HvcDir "mcu_control.in"),pci-address=00:11.0",
        "--serial", "hardware=legacy-virtio-console,num=16,type=file,path=$(Join-Path $LogDir "mcu-uart-hvc15.txt"),input=$(Join-Path $HvcDir "mcu_uart.in"),pci-address=00:12.0"
    )
}

$crosvmArgs = @(
    "--log-level", "info",
    $RunMode,
    "--disable-sandbox",
    "--cid", "$Cid",
    "--mem", "$Mem",
    "--cpus", "$Cpus",
    "--no-balloon",
    "--no-usb",
    "--gpu", $GpuArg,
    "--block", "path=$Disk,ro=false,lock=false,sparse=false,pci-address=00:03.0"
) + $serialArgs + @(
    "--android-fstab", $Fstab,
    "--initrd", $Initrd,
    "--params", $KernelParams,
    $Kernel
)

$commandFile = Join-Path $LogDir "crosvm-command.txt"
@(
    "ArtifactDir=$ArtifactDir",
    "WorkDir=$WorkDir",
    "LogDir=$LogDir",
    "GFXSTREAM_ANGLE_ROOT=$env:GFXSTREAM_ANGLE_ROOT",
    "VK_ICD_FILENAMES=$env:VK_ICD_FILENAMES",
    "CROSVM_WHPX_BLOCK_QUEUES=$env:CROSVM_WHPX_BLOCK_QUEUES",
    "Initrd=$Initrd",
    "",
    $Crosvm,
    ($crosvmArgs | ForEach-Object { "  $_" })
) | Set-Content -Path $commandFile -Encoding ASCII

Write-Host "ArtifactDir: $ArtifactDir"
Write-Host "WorkDir:     $WorkDir"
Write-Host "LogDir:      $LogDir"
Write-Host "Command:     $commandFile"
Write-Host "ANGLE:       $env:GFXSTREAM_ANGLE_ROOT"
if ($env:VK_ICD_FILENAMES) { Write-Host "Vulkan ICD:  $env:VK_ICD_FILENAMES" }
Write-Host "CPU/Mem:     $Cpus vCPU, ${Mem}MiB"
Write-Host "Run mode:    $RunMode"
Write-Host "Initrd:      $Initrd"
Write-Host "GPU:         $GpuArg"

if ($DryRun -or $NoRun) {
    exit 0
}

$stdout = Join-Path $LogDir "stdout.txt"
$stderr = Join-Path $LogDir "stderr.txt"
Remove-Item -LiteralPath $stdout, $stderr -Force -ErrorAction SilentlyContinue

Write-Host "Launching Android on crosvm; logs are under $LogDir"
if ($TimeoutSecs -gt 0) {
    $argumentLine = Join-WindowsArgumentList -Arguments $crosvmArgs
    $process = Start-Process -FilePath $Crosvm -ArgumentList $argumentLine -PassThru `
        -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    $deadline = (Get-Date).AddSeconds($TimeoutSecs)
    while (-not $process.HasExited -and (Get-Date) -lt $deadline) {
        Start-Sleep -Seconds 1
    }
    if (-not $process.HasExited) {
        Write-Host "Timeout reached (${TimeoutSecs}s); stopping crosvm pid $($process.Id)"
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        exit 124
    }
    exit $process.ExitCode
}

& $Crosvm @crosvmArgs > $stdout 2> $stderr
exit $LASTEXITCODE
