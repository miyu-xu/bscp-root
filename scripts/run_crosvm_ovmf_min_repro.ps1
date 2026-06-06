# Minimal WHPX firmware repro: OVMF only, no disk, no virtmgr.
# Logs to out/dist/logs/whpx-ovmf-min-repro/ for WHPX IO classification.
param(
    [string]$LogDir = "",
    [int]$TimeoutSecs = 25,
    [int]$MemMiB = 512,
    [string]$LogLevel = "warn,whpx=debug"
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$FwDir = Join-Path $RepoRoot "out\dist\firmware"
$BinDir = Join-Path $RepoRoot "out\dist\windows\bin"
$Crosvm = Join-Path $BinDir "crosvm.exe"
$Ovmf = Join-Path $FwDir "OVMF.fd"

if (-not $LogDir) {
    $LogDir = Join-Path $RepoRoot "out\dist\logs\whpx-ovmf-min-repro"
}
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null

$SerialLog = Join-Path $LogDir "guest-serial-num1.txt"
$StderrLog = Join-Path $LogDir "crosvm-stderr.txt"
$StdoutLog = Join-Path $LogDir "crosvm-stdout.txt"
$SummaryLog = Join-Path $LogDir "summary.txt"

foreach ($p in @($Crosvm, $Ovmf)) {
    if (-not (Test-Path $p)) { throw "Missing: $p" }
}

$env:PATH = "$BinDir;$env:PATH"
$env:CROSWVM_WHPX_IO_DEBUG = "1"

$crosvmArgs = @(
    "--log-level", $LogLevel,
    "run",
    "--disable-sandbox",
    "--cid", "4200",
    "--mem", $MemMiB.ToString(),
    "--cpus", "1",
    "--no-usb",
    "--no-balloon",
    "--serial", "type=file,path=$SerialLog,hardware=serial,num=1,earlycon=true",
    "--serial", "type=sink,hardware=serial,num=2",
    "--bios", $Ovmf
)

@"
=== WHPX OVMF minimal repro ===
Time: $(Get-Date -Format o)
Crosvm: $Crosvm
OVMF: $Ovmf
MemMiB: $MemMiB
TimeoutSecs: $TimeoutSecs
LogLevel: $LogLevel
Args:
$($crosvmArgs -join ' ')
"@ | Set-Content -Path $SummaryLog -Encoding utf8

Write-Host "LogDir: $LogDir"
Write-Host "Start:  $(Get-Date -Format 'HH:mm:ss')"

$proc = Start-Process -FilePath $Crosvm -ArgumentList $crosvmArgs `
    -RedirectStandardError $StderrLog -RedirectStandardOutput $StdoutLog `
    -PassThru -NoNewWindow

$deadline = (Get-Date).AddSeconds($TimeoutSecs)
while (-not $proc.HasExited -and (Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 2
}
if (-not $proc.HasExited) {
    Write-Host "Timeout ${TimeoutSecs}s — stopping pid=$($proc.Id)"
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep 1
}

$exitCode = $proc.ExitCode
Add-Content -Path $SummaryLog -Value "End: $(Get-Date -Format o)`nExitCode: $exitCode"

Write-Host "End:    $(Get-Date -Format 'HH:mm:ss') exit=$exitCode"
Get-Item $SerialLog, $StderrLog, $StdoutLog -ErrorAction SilentlyContinue |
    ForEach-Object { Write-Host "$($_.Name): $($_.Length) bytes" }

# Classify WHPX IO debug lines if present
$whpxLines = Select-String -Path $StderrLog -Pattern "whpx io emulation failed" -ErrorAction SilentlyContinue
if ($whpxLines) {
    $ports = $whpxLines | ForEach-Object {
        if ($_.Line -match 'port=0x([0-9a-fA-F]+)') { $matches[1] }
    } | Where-Object { $_ } | Group-Object | Sort-Object Count -Descending
    "`n=== Top PIO ports (from whpx debug) ===" | Add-Content $SummaryLog
    $ports | Select-Object -First 20 | ForEach-Object {
        "port=0x$($_.Name) count=$($_.Count)" | Add-Content $SummaryLog
        Write-Host "  port=0x$($_.Name) count=$($_.Count)"
    }
} else {
    $ioErrors = (Select-String -Path $StderrLog -Pattern "failed to handle io" -ErrorAction SilentlyContinue | Measure-Object).Count
    "failed to handle io lines: $ioErrors" | Add-Content $SummaryLog
    Write-Host "failed to handle io lines: $ioErrors (no whpx debug lines — rebuild crosvm with instrumentation)"
}

Get-Content $SummaryLog
