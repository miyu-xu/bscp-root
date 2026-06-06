# ChromiumOS priority boot: OVMF firmware + disk (depthcharge / vboot chain).
# Uses virtmgr + vm.exe when -Direct is not set; otherwise calls crosvm directly.
param(
    [switch]$Direct,
    [switch]$Gfx,
    [ValidateSet("ovmf", "ovmf-debug", "ovmf-split", "seabios")]
    [string]$Firmware = "ovmf",
    [string]$LogDir = "",
    [int]$TimeoutSecs = 120,
    [switch]$SkipHypervisorCheck
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$Config = Join-Path $RepoRoot "scripts\chromiumos_windows_firmware.json"
$FwDir = Join-Path $RepoRoot "out\dist\firmware"

if (-not (Test-Path $Config)) { throw "Missing config: $Config" }
if (-not (Test-Path (Join-Path $FwDir "OVMF.fd"))) {
    throw "OVMF firmware missing. See doc/CHROMIUMOS_FIRMWARE.md for Debian .deb extraction."
}

if ($Direct -or $Gfx) {
    $fwArgs = @{
        Firmware = $Firmware
        LogDir = $LogDir
        TimeoutSecs = $TimeoutSecs
    }
    if ($Gfx) { $fwArgs.UseGpu = $true }
    & (Join-Path $RepoRoot "scripts\run_crosvm_chromeos_firmware.ps1") @fwArgs
    exit $LASTEXITCODE
}

$guestLog = Join-Path $RepoRoot "out\dist\logs\chromeos-firmware-guest.log"
if ($LogDir) {
    New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
}
$vmScript = Join-Path $RepoRoot "scripts\vm_windows.ps1"

$env:VIRTMGR_CAPTURE_GUEST_CONSOLE = "1"
$env:VIRTMGR_CAPTURE_CROSVM_STDIO = "1"
$env:PATH = "C:\workspace\bscp\angle\out\Release-GfxAngle-Clang;$RepoRoot\out\dist\windows\bin;$env:PATH"

Write-Host "Priority boot: virtmgr + OVMF + disk"
Write-Host "Config: $Config"
if ($SkipHypervisorCheck) {
    if ($LogDir) {
        & $vmScript -Command run -Config $Config -Console ttyS0 -GuestLog $guestLog -SkipHypervisorCheck -WorkDir $LogDir
    } else {
        & $vmScript -Command run -Config $Config -Console ttyS0 -GuestLog $guestLog -SkipHypervisorCheck
    }
} elseif ($LogDir) {
    & $vmScript -Command run -Config $Config -Console ttyS0 -GuestLog $guestLog -WorkDir $LogDir
} else {
    & $vmScript -Command run -Config $Config -Console ttyS0 -GuestLog $guestLog
}
