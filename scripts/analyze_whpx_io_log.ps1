# Parse crosvm stderr for WHPX IO emulation diagnostics.
param(
    [Parameter(Mandatory = $true)]
    [string]$StderrLog,
    [string]$OutFile = ""
)

$ErrorActionPreference = "Stop"
if (-not (Test-Path $StderrLog)) { throw "Not found: $StderrLog" }

$whpx = Select-String -Path $StderrLog -Pattern "whpx (io|mmio) emulation failed" -ErrorAction SilentlyContinue
$genericIo = Select-String -Path $StderrLog -Pattern "failed to handle io:" -ErrorAction SilentlyContinue

$report = [ordered]@{
    WhpxDetailLines = $whpx.Count
    GenericIoErrors = $genericIo.Count
    Ports = @()
    Mmio = @()
    StatusCodes = @()
}

if ($whpx) {
    $report.Ports = $whpx | Where-Object { $_.Line -match 'port=0x' } | ForEach-Object {
        $m = [regex]::Match($_.Line, 'port=0x([0-9a-fA-F]+).*dir=(\w+).*size=(\d+).*rip=0x([0-9a-fA-F]+)')
        if ($m.Success) {
            [pscustomobject]@{
                Port = "0x$($m.Groups[1].Value)"
                Dir = $m.Groups[2].Value
                Size = $m.Groups[3].Value
                Rip = "0x$($m.Groups[4].Value)"
                Line = $_.Line
            }
        }
    } | Group-Object Port | ForEach-Object {
        [pscustomobject]@{ Port = $_.Name; Count = $_.Count; Sample = $_.Group[0].Line }
    } | Sort-Object Count -Descending

    $report.Mmio = $whpx | Where-Object { $_.Line -match 'gpa=0x' } | ForEach-Object {
        $m = [regex]::Match($_.Line, 'gpa=0x([0-9a-fA-F]+)')
        if ($m.Success) { $m.Groups[1].Value }
    } | Group-Object | Sort-Object Count -Descending | Select-Object -First 15

    $report.StatusCodes = $whpx | ForEach-Object {
        if ($_.Line -match 'status_as_u32=(\d+)') { [int]$matches[1] }
    } | Group-Object | Sort-Object Count -Descending
}

$text = $report | ConvertTo-Json -Depth 5
if ($OutFile) {
    $text | Set-Content -Path $OutFile -Encoding utf8
    Write-Host "Wrote $OutFile"
}
$text
