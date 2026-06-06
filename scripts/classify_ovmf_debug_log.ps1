# Heuristic classification of OVMF DEBUG serial/debugcon logs.
param(
    [string]$LogDir = ""
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
if (-not $LogDir) { $LogDir = Join-Path $RepoRoot "out\dist\logs\cros-fw-ovmf-debug" }

$serial = Join-Path $LogDir "guest-serial-num1.txt"
$debugcon = Join-Path $LogDir "guest-debugcon.txt"
$summary = Join-Path $LogDir "firmware-stage.txt"

function Read-LogText($path) {
    if (-not (Test-Path $path)) { return "" }
    $len = (Get-Item $path).Length
    if ($len -eq 0) { return "" }
    return Get-Content $path -Raw -Encoding byte -ErrorAction SilentlyContinue
}

# Try UTF-8 / ASCII read
function Read-LogAscii($path) {
    if (-not (Test-Path $path)) { return @() }
    if ((Get-Item $path).Length -eq 0) { return @() }
    try { return Get-Content $path -Encoding utf8 } catch { return Get-Content $path }
}

$text = (@(Read-LogAscii $serial) + @(Read-LogAscii $debugcon)) -join "`n"
$szSerial = if (Test-Path $serial) { (Get-Item $serial).Length } else { 0 }
$szDbg = if (Test-Path $debugcon) { (Get-Item $debugcon).Length } else { 0 }

$stage = "unknown"
$notes = @()

if ($szSerial -eq 0 -and $szDbg -eq 0) {
    $stage = "no_firmware_output"
    $notes += "No bytes on serial or debugcon — check OVMF_DEBUG.fd, WHPX, or crosvm serial wiring."
} else {
    $notes += "serial=$szSerial bytes debugcon=$szDbg bytes"
    if ($text -match '(?i)SEC Phase') { $notes += "SEC phase seen" }
    if ($text -match '(?i)PEI') { $notes += "PEI seen" }
    if ($text -match '(?i)DXE') { $notes += "DXE seen" }
    if ($text -match '(?i)BDS') { $notes += "BDS seen" }
    if ($text -match '(?i)Boot Manager|BootOrder|Boot000') { $notes += "UEFI Boot Manager activity" }
    if ($text -match '(?i)virtio|Virtio|VIRTIO') { $notes += "Virtio block/network mentioned" }
    if ($text -match '(?i)EFI-SYSTEM|SYSLINUX|depthcharge|ChromeOS') {
        $notes += "ChromeOS EFI / depthcharge path mentioned"
    }
    if ($text -match '(?i)ASSERT') { $notes += "ASSERT/firmware trap — likely WHPX IO/MMIO alignment or unimplemented port" }
    if ($text -match 'IoLibGcc\.c.*Port & 3') { $notes += "Unaligned IO port access in PEI (IoLibGcc)" }

    if ($text -match '(?i)ASSERT') { $stage = "pei_assert_io_alignment" }
    elseif ($text -match '(?i)BDS') { $stage = "reached_bds" }
    elseif ($text -match '(?i)DXE') { $stage = "reached_dxe" }
    elseif ($text -match '(?i)PEI') { $stage = "reached_pei" }
    elseif ($text -match '(?i)SEC') { $stage = "reached_sec" }
    else { $stage = "firmware_output_unclassified" }

    if ($text -match '(?i)virtio|Virtio') { $stage = "disk_enumeration_likely" }
    if ($text -match '(?i)SYSLINUX|depthcharge') { $stage = "depthcharge_handoff_likely" }
}

$report = @"
=== OVMF DEBUG firmware stage ===
LogDir: $LogDir
Stage: $stage
$(($notes | ForEach-Object { "- $_" }) -join "`n")

--- serial (first 40 lines) ---
$((Read-LogAscii $serial | Select-Object -First 40) -join "`n")

--- debugcon (first 20 lines) ---
$((Read-LogAscii $debugcon | Select-Object -First 20) -join "`n")
"@

Set-Content -Path $summary -Value $report -Encoding utf8
Write-Host $report
return $stage
