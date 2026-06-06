# Fetch OVMF/SeaBIOS firmware from Debian packages (no WSL).
# Release OVMF logs to debugcon port 0x402 (use -UseDebugcon in run scripts).
# For DEBUG_ON_SERIAL_PORT OVMF you must build edk2 from source (see doc/CHROMIUMOS_FIRMWARE_VISIBILITY.md).
param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$FwDir = Join-Path $RepoRoot "out\dist\firmware"
New-Item -ItemType Directory -Force -Path $FwDir | Out-Null

$ovmfDeb = Join-Path $FwDir "ovmf.deb"
$seabiosDeb = Join-Path $FwDir "seabios.deb"
$ovmfUrl = "http://ftp.debian.org/debian/pool/main/o/ovmf/ovmf_2025.02-9_all.deb"
$seabiosUrl = "http://ftp.debian.org/debian/pool/main/s/seabios/seabios_1.16.3-2_all.deb"

function Fetch-Deb($url, $out) {
    if ((Test-Path $out) -and -not $Force) { return }
    Write-Host "Downloading $url"
    Invoke-WebRequest -Uri $url -OutFile $out
}

Fetch-Deb $ovmfUrl $ovmfDeb
Fetch-Deb $seabiosUrl $seabiosDeb
tar -xf $ovmfDeb -C $FwDir
tar -xf $seabiosDeb -C $FwDir

$map = @(
    @("$FwDir\usr\share\OVMF\OVMF.fd", "$FwDir\OVMF.fd"),
    @("$FwDir\usr\share\OVMF\OVMF_CODE.fd", "$FwDir\OVMF_CODE.fd"),
    @("$FwDir\usr\share\OVMF\OVMF_VARS.fd", "$FwDir\OVMF_VARS.fd"),
    @("$FwDir\usr\share\seabios\bios.bin", "$FwDir\bios.bin")
)
foreach ($m in $map) {
    if (Test-Path $m[0]) { Copy-Item $m[0] $m[1] -Force }
}

Write-Host "Firmware in $FwDir"
Get-ChildItem $FwDir -File | Where-Object { $_.Extension -in ".fd", ".bin" } | Format-Table Name, Length -AutoSize
