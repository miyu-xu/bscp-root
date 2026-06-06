# Patch EFI-SYSTEM partition on a disk image copy for virtio-disk boot.
param(
    [string]$SrcImage = "",
    [string]$DstImage = "",
    [ValidateSet("vhd", "hd", "vusb")]
    [string]$BootTarget = "vhd"
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
if (-not $SrcImage) { $SrcImage = Join-Path $RepoRoot "out\dist\img\amd64-generic_test_image.bin" }
if (-not $DstImage) {
    $suffix = if ($BootTarget -eq "vusb") { "orig" } else { $BootTarget }
    $DstImage = Join-Path $RepoRoot "out\dist\img\amd64-generic_test_image_$suffix.bin"
}

$efiStart = [int64]0x6D000 * 512L
$efiSize = [int64]0x40000 * 512L

$labelMap = @{
    vhd  = @{ Old = "chromeos-vusb.A"; New = "chromeos-vhd.A " }
    hd   = @{ Old = "chromeos-vusb.A"; New = "chromeos-hd.A  " }
    vusb = @{ Old = "chromeos-vhd.A "; New = "chromeos-vusb.A" }
}
if (-not $labelMap.ContainsKey($BootTarget)) { throw "Unknown BootTarget: $BootTarget" }

function Patch-Bytes([byte[]]$hay, [byte[]]$old, [byte[]]$new) {
    if ($old.Length -ne $new.Length) { throw "patch length mismatch ($($old.Length) vs $($new.Length))" }
    $count = 0
    for ($i = 0; $i -le $hay.Length - $old.Length; $i++) {
        $match = $true
        for ($j = 0; $j -lt $old.Length; $j++) {
            if ($hay[$i + $j] -ne $old[$j]) { $match = $false; break }
        }
        if ($match) {
            for ($j = 0; $j -lt $new.Length; $j++) { $hay[$i + $j] = $new[$j] }
            $count++
        }
    }
    return $count
}

Write-Host "Copy $SrcImage -> $DstImage"
Copy-Item $SrcImage $DstImage -Force

$efi = New-Object byte[] $efiSize
$fs = [IO.File]::Open($DstImage, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
try {
    $fs.Seek($efiStart, [IO.SeekOrigin]::Begin) | Out-Null
    $read = 0
    while ($read -lt $efiSize) {
        $n = $fs.Read($efi, $read, [int]($efiSize - $read))
        if ($n -le 0) { throw "short read at EFI partition" }
        $read += $n
    }

    $txt = [Text.Encoding]::ASCII.GetString($efi)
    if ($BootTarget -ne "vusb" -and $txt.IndexOf("chromeos-vusb.A") -lt 0) {
        throw "chromeos-vusb.A not found in EFI partition"
    }

    $oldLabel = [Text.Encoding]::ASCII.GetBytes($labelMap[$BootTarget].Old)
    $newLabel = [Text.Encoding]::ASCII.GetBytes($labelMap[$BootTarget].New)
    $nLabel = Patch-Bytes $efi $oldLabel $newLabel
    Write-Host "Patched $nLabel label(s): $($labelMap[$BootTarget].Old) -> '$($labelMap[$BootTarget].New)' (BootTarget=$BootTarget)"

    if ($BootTarget -ne "vusb") {
        $oldModeset = [Text.Encoding]::ASCII.GetBytes("i915.modeset=1")
        $newConsole = [Text.Encoding]::ASCII.GetBytes("console=ttyS0 ")
        $nConsole = Patch-Bytes $efi $oldModeset $newConsole
        Write-Host "Patched $nConsole i915.modeset=1 -> console=ttyS0"

        $oldEarly = [Text.Encoding]::ASCII.GetBytes("loglevel=7 ")
        $newEarly = [Text.Encoding]::ASCII.GetBytes("earlycon=s0")
        $nEarly = Patch-Bytes $efi $oldEarly $newEarly
        Write-Host "Patched $nEarly loglevel=7 -> earlycon=s0"
    }

    $fs.Seek($efiStart, [IO.SeekOrigin]::Begin) | Out-Null
    $fs.Write($efi, 0, $efi.Length)
} finally {
    $fs.Close()
}
Write-Host "Wrote $DstImage"
