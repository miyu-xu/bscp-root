# Read first sectors of EFI-SYSTEM partition (SYSLINUX / depthcharge hints).
param(
    [string]$ImagePath = "",
    [int]$ReadBytes = 4096
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
if (-not $ImagePath) {
    $ImagePath = Join-Path $RepoRoot "out\dist\img\amd64-generic_test_image.bin"
}
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
    $partEntryLba = [BitConverter]::ToInt64($hdr, 72)
    $numEntries = [BitConverter]::ToUInt32($hdr, 80)
    $entrySize = [BitConverter]::ToUInt32($hdr, 84)
    $table = New-Object byte[] ($numEntries * $entrySize)
    $fs.Seek($partEntryLba * $LBA, [System.IO.SeekOrigin]::Begin) | Out-Null
    $null = $fs.Read($table, 0, $table.Length)

    $start = $null
    for ($i = 0; $i -lt $numEntries; $i++) {
        $o = $i * $entrySize
        $typeStr = GuidToString $table[$o..($o + 15)]
        if ($typeStr -eq $efiGuid) {
            $start = [BitConverter]::ToInt64($table, $o + 32)
            break
        }
    }
    if (-not $start) { throw "EFI-SYSTEM partition not found" }
    $offset = $start * $LBA
    Write-Host "EFI-SYSTEM LBA start=$start offset=$offset bytes"
    $buf = New-Object byte[] $ReadBytes
    $fs.Seek($offset, [System.IO.SeekOrigin]::Begin) | Out-Null
    $n = $fs.Read($buf, 0, $ReadBytes)
    $ascii = [Text.Encoding]::ASCII.GetString($buf) -replace '[^\x20-\x7E\r\n]', '.'
    Write-Host "`nFirst $n bytes (ASCII):"
    Write-Host $ascii.Substring(0, [Math]::Min(512, $ascii.Length))
    if ($buf[0] -eq 0xEB -or ($buf[0] -eq 0xE9)) { Write-Host "`nLooks like x86 boot sector / FAT BPB" }
    $fatSig = [Text.Encoding]::ASCII.GetString($buf, 510, 2)
    if ($fatSig -eq "U") { Write-Host "FAT boot signature 0x55AA at sector 0" }
} finally {
    $fs.Dispose()
}
