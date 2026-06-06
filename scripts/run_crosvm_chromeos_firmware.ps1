# Boot ChromiumOS disk via prebuilt firmware (SeaBIOS or OVMF UEFI) — no direct kernel.
param(
    [ValidateSet("seabios", "ovmf", "ovmf-debug", "ovmf-debug-debugcon", "ovmf-split")]
    [string]$Firmware = "ovmf",
    [string]$LogDir = "",
    [int]$TimeoutSecs = 120,
    [int]$MemMiB = 4096,
    [string]$Image = "",
    [switch]$NoDebugcon,
    [switch]$UseGpu,
    [ValidateSet("", "2d", "gfxstream")]
    [string]$GpuBackend = "",
    [switch]$EnableUsb,
    [switch]$BlockReadOnly
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$FwDir = Join-Path $RepoRoot "out\dist\firmware"
$BinDir = Join-Path $RepoRoot "out\dist\windows\bin"
$Img = if ($Image) {
    if ([System.IO.Path]::IsPathRooted($Image)) { $Image } else { Join-Path $RepoRoot $Image }
} else {
    Join-Path $RepoRoot "out\dist\img\amd64-generic_test_image.bin"
}
$Crosvm = Join-Path $BinDir "crosvm.exe"

if (-not $LogDir) { $LogDir = Join-Path $RepoRoot "out\dist\logs\cros-fw-$Firmware" }
elseif (-not [System.IO.Path]::IsPathRooted($LogDir)) {
    $LogDir = Join-Path $RepoRoot $LogDir
}
$LogDir = [System.IO.Path]::GetFullPath($LogDir)
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$SerialLog = Join-Path $LogDir "guest-serial-num1.txt"
$VirtioConLog = Join-Path $LogDir "guest-virtio-console1.txt"
$DebugconLog = Join-Path $LogDir "guest-debugcon.txt"
$CrosvmErr = Join-Path $LogDir "crosvm-stderr.txt"
$CrosvmOut = Join-Path $LogDir "crosvm-stdout.txt"
$SummaryLog = Join-Path $LogDir "summary.txt"

foreach ($p in @($Crosvm, $Img)) { if (-not (Test-Path $p)) { throw "Missing: $p" } }

$env:PATH = "C:\workspace\bscp\angle\out\Release-GfxAngle-Clang;$BinDir;$env:PATH"

$crosvmArgs = @(
    "--log-level", "info,disk=warn",
    "run",
    "--disable-sandbox",
    "--cid", "4098",
    "--mem", $MemMiB.ToString(),
    "--cpus", "1",
    "--no-balloon",
    "--serial", "type=file,path=$SerialLog,hardware=serial,num=1,earlycon=true",
    "--serial", "type=sink,hardware=serial,num=2",
    "--serial", "type=file,path=$VirtioConLog,hardware=virtio-console,num=1,console=true",
    "--serial", "type=sink,hardware=virtio-console,num=2"
)

if (-not $EnableUsb) {
    $crosvmArgs += "--no-usb"
}

if (-not $NoDebugcon) {
    # OVMF DEBUG/development builds write firmware logs to ISA debugcon port 0x402.
    $crosvmArgs += @("--serial", "type=file,path=$DebugconLog,hardware=debugcon,num=3,debugcon_port=402")
}

if ($GpuBackend -eq "2d") {
    $crosvmArgs += @("--gpu", "backend=2d,width=1024,height=768")
} elseif ($UseGpu -or $GpuBackend -eq "gfxstream") {
    $crosvmArgs += @(
        "--gpu", "backend=gfxstream",
        "--display-window-keyboard",
        "--display-window-mouse"
    )
}

$blockRo = if ($BlockReadOnly) { "true" } else { "false" }
$crosvmArgs += @("--block", "path=$Img,ro=$blockRo,lock=false,sparse=false,bootindex=1")

switch ($Firmware) {
    "seabios" {
        $bios = Join-Path $FwDir "bios.bin"
        if (-not (Test-Path $bios)) { throw "SeaBIOS not found: $bios" }
        $crosvmArgs += @("--bios", $bios)
    }
    "ovmf" {
        $ovmf = Join-Path $FwDir "OVMF.fd"
        if (-not (Test-Path $ovmf)) { throw "OVMF not found: $ovmf" }
        $crosvmArgs += @("--bios", $ovmf)
    }
    "ovmf-debug" {
        $ovmf = Join-Path $FwDir "OVMF_DEBUG.fd"
        if (-not (Test-Path $ovmf)) {
            throw @"
OVMF_DEBUG.fd not found. Build edk2 with DEBUG_ON_SERIAL_PORT:
  .\scripts\build_ovmf_debug.ps1
See doc/OVMF_DEBUG_BUILD.md
"@
        }
        $crosvmArgs += @("--bios", $ovmf)
    }
    "ovmf-debug-debugcon" {
        $ovmf = Join-Path $FwDir "OVMF_DEBUG_DEBUGCON.fd"
        if (-not (Test-Path $ovmf)) {
            throw @"
OVMF_DEBUG_DEBUGCON.fd not found. Build with:
  wsl bash -lc 'cd /mnt/c/workspace/bscp/bscp && OVMF_SERIAL_DEBUG=0 scripts/build_ovmf_debug.sh'
"@
        }
        $crosvmArgs += @("--bios", $ovmf)
    }
    "ovmf-split" {
        $code = Join-Path $FwDir "OVMF_CODE.fd"
        $varsSrc = Join-Path $FwDir "OVMF_VARS.fd"
        $varsCopy = Join-Path $LogDir "OVMF_VARS.fd"
        Copy-Item $varsSrc $varsCopy -Force
        $crosvmArgs += @("--bios", $code, "--pflash", "path=$varsCopy,block_size=4096")
    }
}

@"
=== ChromeOS firmware boot ===
Time: $(Get-Date -Format o)
Firmware: $Firmware
Debugcon: $(-not $NoDebugcon)
GPU: $(if ($GpuBackend) { $GpuBackend } elseif ($UseGpu) { 'gfxstream' } else { 'none' })
Args: $($crosvmArgs -join ' ')
"@ | Set-Content $SummaryLog -Encoding utf8

Write-Host "Firmware : $Firmware"
Write-Host "LogDir   : $LogDir"
Write-Host "Debugcon : $(-not $NoDebugcon)"
Write-Host "Start    : $(Get-Date -Format 'HH:mm:ss')"

$proc = Start-Process -FilePath $Crosvm -ArgumentList $crosvmArgs `
    -RedirectStandardError $CrosvmErr -RedirectStandardOutput $CrosvmOut `
    -PassThru -NoNewWindow

$deadline = (Get-Date).AddSeconds($TimeoutSecs)
while (-not $proc.HasExited -and (Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 2
}
if (-not $proc.HasExited) {
    Write-Host "Timeout ${TimeoutSecs}s — stopping pid=$($proc.Id)"
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
}

Add-Content $SummaryLog "`nEnd: $(Get-Date -Format o) exit=$($proc.ExitCode)"
Write-Host "End      : $(Get-Date -Format 'HH:mm:ss') exit=$($proc.ExitCode)"

foreach ($f in @($SerialLog, $VirtioConLog, $DebugconLog, $CrosvmErr)) {
    if (Test-Path $f) {
        $sz = (Get-Item $f).Length
        $line = "$([IO.Path]::GetFileName($f)): $sz bytes"
        Add-Content $SummaryLog $line
        Write-Host $line
        if ($sz -gt 0 -and $sz -lt 50000) { Get-Content $f -TotalCount 15 }
        elseif ($sz -gt 0) { Get-Content $f -TotalCount 8; Write-Host "  ..." }
    }
}

Get-Content $SummaryLog
