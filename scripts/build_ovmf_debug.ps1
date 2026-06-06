# Build OVMF_DEBUG.fd via WSL (Ubuntu) or native bash.
param(
    [switch]$Force,
    [string]$WslDistro = ""
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$OutFd = Join-Path $RepoRoot "out\dist\firmware\OVMF_DEBUG.fd"

if ((Test-Path $OutFd) -and -not $Force) {
    Write-Host "Already exists: $OutFd ($((Get-Item $OutFd).Length) bytes)"
    exit 0
}

$wslPath = "/mnt/c" + ($RepoRoot -replace '\\', '/' -replace '^C:', '')
$bashScript = "$wslPath/scripts/build_ovmf_debug.sh"

$wslArgs = @()
if ($WslDistro) { $wslArgs += @("-d", $WslDistro) }
$wslArgs += @("bash", "-lc", "cd '$wslPath' && chmod +x scripts/build_ovmf_debug.sh && scripts/build_ovmf_debug.sh")

Write-Host "Building OVMF_DEBUG.fd in WSL..."
Write-Host "  Repo: $wslPath"
& wsl @wslArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (-not (Test-Path $OutFd)) { throw "Build finished but missing: $OutFd" }
Get-Item $OutFd | Format-List FullName, Length, LastWriteTime
