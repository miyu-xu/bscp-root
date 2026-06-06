# List ChromeOS GPT partitions and EFI-SYSTEM offset (for firmware boot debugging).
param(
    [string]$ImagePath = ""
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
if (-not $ImagePath) {
    $ImagePath = Join-Path $RepoRoot "out\dist\img\amd64-generic_test_image.bin"
}
if (-not (Test-Path $ImagePath)) { throw "Image not found: $ImagePath" }

$efiGuid = "C12A7328-F81F-11D2-BA4B-00A0C93EC93B"
$LBA = 512

function GuidToString([byte[]]$g) {
    return ("{0:X2}{1:X2}{2:X2}{3:X2}-{4:X2}{5:X2}-{6:X2}{7:X2}-{8:X2}{9:X2}-{10:X2}{11:X2}{12:X2}{13:X2}{14:X2}{15:X2}" -f
        $g[3],$g[2],$g[1],$g[0], $g[5],$g[4], $g[7],$g[6], $g[8],$g[9],
        $g[10],$g[11],$g[12],$g[13],$g[14],$g[15])
}

$fs = [System.IO.File]::OpenRead($ImagePath)
try {
    $hdr = New-Object byte[] 92
    $fs.Seek(512, [System.IO.SeekOrigin]::Begin) | Out-Null
    $null = $fs.Read($hdr, 0, 92)
    if ([Text.Encoding]::ASCII.GetString($hdr, 0, 8) -ne "EFI PART") { throw "Not a GPT image" }
    $partEntryLba = [BitConverter]::ToInt64($hdr, 72)
    $numEntries = [BitConverter]::ToUInt32($hdr, 80)
    $entrySize = [BitConverter]::ToUInt32($hdr, 84)
    $table = New-Object byte[] ($numEntries * $entrySize)
    $fs.Seek($partEntryLba * $LBA, [System.IO.SeekOrigin]::Begin) | Out-Null
    $null = $fs.Read($table, 0, $table.Length)

    $rows = @()
    for ($i = 0; $i -lt $numEntries; $i++) {
        $o = $i * $entrySize
        $empty = $true
        for ($j = 0; $j -lt 16; $j++) { if ($table[$o + $j] -ne 0) { $empty = $false; break } }
        if ($empty) { continue }
        $typeGuid = $table[$o..($o + 15)]
        $start = [BitConverter]::ToInt64($table, $o + 32)
        $end = [BitConverter]::ToInt64($table, $o + 40)
        $name = [Text.Encoding]::Unicode.GetString($table, $o + 56, 72).TrimEnd([char]0)
        $typeStr = GuidToString $typeGuid
        $rows += [pscustomobject]@{
            Index = $i + 1
            Name = $name
            TypeGuid = $typeStr
            IsEfi = ($typeStr -eq $efiGuid)
            StartMB = [math]::Round($start * $LBA / 1MB, 2)
            SizeMB = [math]::Round(($end - $start + 1) * $LBA / 1MB, 2)
        }
    }
    $rows | Format-Table -AutoSize
    $efi = $rows | Where-Object { $_.IsEfi -or $_.Name -match "EFI" }
    if ($efi) {
        Write-Host "`nEFI / boot-related partitions:"
        $efi | Format-Table -AutoSize
    } else {
        Write-Host "`nWARNING: No standard EFI partition type found"
    }
} finally {
    $fs.Dispose()
}
