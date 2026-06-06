# Quick probe: does crosvm gfxstream backend load?
param(
    [int]$TimeoutSecs = 8
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$BinDir = Join-Path $RepoRoot "out\dist\windows\bin"
$Crosvm = Join-Path $BinDir "crosvm.exe"
$LogDir = Join-Path $RepoRoot "out\dist\logs\cros-fw-gfx-probe"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Err = Join-Path $LogDir "stderr.txt"

$env:PATH = "C:\workspace\bscp\angle\out\Release-GfxAngle-Clang;$BinDir;$env:PATH"

$args = @(
    "run", "--disable-sandbox", "--mem", "512", "--cpus", "1",
    "--bios", (Join-Path $RepoRoot "out\dist\firmware\OVMF.fd"),
    "--gpu", "backend=gfxstream",
    "--display-window-keyboard", "--display-window-mouse"
)

Write-Host "Probing gfxstream (timeout ${TimeoutSecs}s)..."
$p = Start-Process -FilePath $Crosvm -ArgumentList $args -PassThru -NoNewWindow `
    -RedirectStandardError $Err -RedirectStandardOutput (Join-Path $LogDir "stdout.txt")
$deadline = (Get-Date).AddSeconds($TimeoutSecs)
while (-not $p.HasExited -and (Get-Date) -lt $deadline) { Start-Sleep 1 }
if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }

Write-Host "--- stderr ---"
if (Test-Path $Err) { Get-Content $Err }
$dll = Join-Path $BinDir "gfxstream_backend.dll"
Write-Host "`ngfxstream_backend.dll in dist: $(Test-Path $dll)"
