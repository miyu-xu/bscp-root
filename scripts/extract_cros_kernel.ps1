param(
    [Parameter(Mandatory=$true)]
    [string]$ImagePath,
    [string]$OutKernel = "",
    [string]$OutRootInfo = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$LBA = 512
$CROS_KERN_GUID_BYTES = [byte[]](
    0x5D,0x2A,0x3A,0xFE,
    0x32,0x4F,
    0xA7,0x41,
    0xB7,0x25,
    0xAC,0xCC,0x32,0x85,0xA3,0x09
)
$CROS_ROOTFS_GUID_BYTES = [byte[]](
    0x02,0xE2,0xB8,0x3C,
    0x7E,0x3B,
    0xDD,0x47,
    0x8A,0x3C,
    0x7F,0xF2,0xA1,0x3C,0xFC,0xEC
)

function Read-Bytes([System.IO.FileStream]$fs, [int64]$offset, [int]$count) {
    $fs.Seek($offset, [System.IO.SeekOrigin]::Begin) | Out-Null
    $buf = New-Object byte[] $count
    $n = $fs.Read($buf, 0, $count)
    if ($n -ne $count) { throw "Short read at $offset (wanted $count, got $n)" }
    return $buf
}

function Cmp-Bytes([byte[]]$a, [int]$ao, [byte[]]$b, [int]$count) {
    for ($i = 0; $i -lt $count; $i++) {
        if ($a[$ao + $i] -ne $b[$i]) { return $false }
    }
    return $true
}

function GuidToString([byte[]]$g) {
    $sb = New-Object System.Text.StringBuilder
    $null = $sb.Append(("{0:X2}{1:X2}{2:X2}{3:X2}-" -f $g[3],$g[2],$g[1],$g[0]))
    $null = $sb.Append(("{0:X2}{1:X2}-" -f $g[5],$g[4]))
    $null = $sb.Append(("{0:X2}{1:X2}-" -f $g[7],$g[6]))
    $null = $sb.Append(("{0:X2}{1:X2}-" -f $g[8],$g[9]))
    for ($i = 10; $i -lt 16; $i++) { $null = $sb.Append(("{0:X2}" -f $g[$i])) }
    return $sb.ToString()
}

if (-not (Test-Path -LiteralPath $ImagePath)) { throw "Image not found: $ImagePath" }
if (-not $OutKernel) { $OutKernel = [System.IO.Path]::ChangeExtension($ImagePath, ".kernel") }
if (-not $OutRootInfo) { $OutRootInfo = [System.IO.Path]::ChangeExtension($ImagePath, ".partinfo.json") }

Write-Host "Image:   $ImagePath"
Write-Host "Kernel:  $OutKernel"
Write-Host "PartInfo: $OutRootInfo"

$fs = [System.IO.File]::Open($ImagePath, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read)
try {
    # Read GPT header at LBA 1
    $hdr = Read-Bytes $fs 512 92
    $sig = [System.Text.Encoding]::ASCII.GetString($hdr, 0, 8)
    if ($sig -ne "EFI PART") { throw "GPT signature mismatch (got '$sig')" }
    $partEntryLba = [BitConverter]::ToInt64($hdr, 72)
    $numEntries   = [BitConverter]::ToUInt32($hdr, 80)
    $entrySize    = [BitConverter]::ToUInt32($hdr, 84)
    Write-Host ("GPT entries: count={0} size={1} at LBA={2}" -f $numEntries, $entrySize, $partEntryLba)

    $partTableOff = $partEntryLba * $LBA
    $partTableBytes = Read-Bytes $fs $partTableOff ($numEntries * $entrySize)

    $partitions = @()
    for ($i = 0; $i -lt $numEntries; $i++) {
        $entryOff = $i * $entrySize
        # Skip empty
        $isEmpty = $true
        for ($j = 0; $j -lt 16; $j++) { if ($partTableBytes[$entryOff + $j] -ne 0) { $isEmpty = $false; break } }
        if ($isEmpty) { continue }

        $typeGuid = $partTableBytes[$entryOff..($entryOff + 15)]
        $startLba = [BitConverter]::ToInt64($partTableBytes, $entryOff + 32)
        $endLba   = [BitConverter]::ToInt64($partTableBytes, $entryOff + 40)
        $sizeBytes = ($endLba - $startLba + 1) * $LBA

        # Name (UTF-16LE, 72 bytes = 36 chars)
        $nameBytes = $partTableBytes[($entryOff + 56)..($entryOff + 127)]
        $name = [System.Text.Encoding]::Unicode.GetString($nameBytes).TrimEnd("`0")

        $isKernel = Cmp-Bytes $typeGuid 0 $CROS_KERN_GUID_BYTES 16
        $isRootfs = Cmp-Bytes $typeGuid 0 $CROS_ROOTFS_GUID_BYTES 16

        $kind = if ($isKernel) { "KERNEL" } elseif ($isRootfs) { "ROOTFS" } else { "" }

        $entryInfo = [pscustomobject]@{
            Index       = $i + 1
            Name        = $name
            Kind        = $kind
            TypeGuid    = (GuidToString $typeGuid)
            StartLba    = $startLba
            EndLba      = $endLba
            StartOffset = $startLba * $LBA
            SizeBytes   = $sizeBytes
        }
        $partitions += $entryInfo
    }

    Write-Host ""
    Write-Host "Partitions:"
    $partitions | Format-Table Index,Name,Kind,StartLba,EndLba,SizeBytes -AutoSize | Out-String | Write-Host

    $kernA = $partitions | Where-Object { $_.Name -eq "KERN-A" -or ($_.Kind -eq "KERNEL" -and $_.Name -match "A") } | Select-Object -First 1
    if (-not $kernA) {
        $kernA = $partitions | Where-Object { $_.Kind -eq "KERNEL" } | Sort-Object Index | Select-Object -First 1
    }
    if (-not $kernA) { throw "No CrOS kernel partition found" }

    $rootA = $partitions | Where-Object { $_.Name -eq "ROOT-A" -or ($_.Kind -eq "ROOTFS" -and $_.Name -match "A") } | Select-Object -First 1
    if (-not $rootA) {
        $rootA = $partitions | Where-Object { $_.Kind -eq "ROOTFS" } | Sort-Object Index | Select-Object -First 1
    }
    if (-not $rootA) { Write-Warning "No CrOS rootfs partition found (will continue, but root= cmdline may be wrong)" }

    Write-Host ("Using kernel partition: Index={0} Name='{1}' offset={2} size={3}" -f $kernA.Index, $kernA.Name, $kernA.StartOffset, $kernA.SizeBytes)
    if ($rootA) { Write-Host ("Using rootfs partition: Index={0} Name='{1}' offset={2} size={3}" -f $rootA.Index, $rootA.Name, $rootA.StartOffset, $rootA.SizeBytes) }

    # Read kernel partition (cap at 32 MiB to keep memory reasonable)
    $kernSize = [int]([Math]::Min($kernA.SizeBytes, 32 * 1024 * 1024))
    Write-Host ("Reading first {0} bytes of KERN-A..." -f $kernSize)
    $kernBytes = Read-Bytes $fs $kernA.StartOffset $kernSize

    # Find Linux x86_64 bzImage magic "HdrS" (4 bytes: 0x48, 0x64, 0x72, 0x53) at +0x202 from bzImage start
    $magic = [byte[]](0x48, 0x64, 0x72, 0x53)
    $hdrSOffset = -1
    for ($i = 0; $i -le $kernBytes.Length - 4; $i++) {
        if ($kernBytes[$i] -eq 0x48 -and $kernBytes[$i+1] -eq 0x64 -and $kernBytes[$i+2] -eq 0x72 -and $kernBytes[$i+3] -eq 0x53) {
            # Verify boot_flag at offset $i - 0x202 + 0x1FE = 0xAA 0x55
            $bootSigOff = $i - 0x202 + 0x1FE
            if ($bootSigOff -ge 0 -and $bootSigOff + 1 -lt $kernBytes.Length) {
                if ($kernBytes[$bootSigOff] -eq 0x55 -and $kernBytes[$bootSigOff + 1] -eq 0xAA) {
                    $hdrSOffset = $i
                    break
                }
            }
        }
    }
    if ($hdrSOffset -lt 0) { throw "Could not find Linux x86_64 bzImage magic 'HdrS' with valid boot sig in KERN-A" }

    $bzImageOffset = $hdrSOffset - 0x202
    Write-Host ("Found HdrS at KERN-A offset 0x{0:X} (bzImage starts at 0x{1:X})" -f $hdrSOffset, $bzImageOffset)

    # Extract bzImage: write from bzImageOffset to end of available data
    $bzImageLen = $kernBytes.Length - $bzImageOffset
    Write-Host ("Writing $bzImageLen bytes of bzImage to $OutKernel")
    [System.IO.File]::WriteAllBytes($OutKernel, $kernBytes[$bzImageOffset..($kernBytes.Length - 1)])

    # Emit partinfo for crosvm/raw-config use
    $info = [pscustomobject]@{
        image_path   = (Resolve-Path $ImagePath).Path
        kernel_path  = (Resolve-Path $OutKernel).Path
        sector_size  = $LBA
        partitions   = $partitions
        kernel_partition = [pscustomobject]@{
            index        = $kernA.Index
            name         = $kernA.Name
            start_offset = $kernA.StartOffset
            size_bytes   = $kernA.SizeBytes
        }
        rootfs_partition = $(if ($rootA) { [pscustomobject]@{
            index        = $rootA.Index
            name         = $rootA.Name
            start_offset = $rootA.StartOffset
            size_bytes   = $rootA.SizeBytes
            guest_device = "/dev/vda{0}" -f $rootA.Index
        } } else { $null })
    }
    $info | ConvertTo-Json -Depth 5 | Set-Content -Path $OutRootInfo -Encoding UTF8
    Write-Host ""
    Write-Host "Wrote: $OutKernel"
    Write-Host "Wrote: $OutRootInfo"
}
finally {
    $fs.Close()
}
