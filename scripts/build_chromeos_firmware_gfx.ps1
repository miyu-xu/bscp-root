# Rebuild crosvm with gfxstream + ANGLE for ChromeOS firmware UI visibility.
# Requires: CMake, MinGW-w64, ANGLE at ..\angle, flatbuffers in external\flatbuffers
$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path

$env:ENABLE_GFXSTREAM_ANGLE = "1"
if (-not $env:AEMU_COMMON_PATH) {
    $inRepo = Join-Path $RepoRoot "hardware\google\aemu"
    if (Test-Path $inRepo) { $env:AEMU_COMMON_PATH = $inRepo }
}
if (-not $env:ANGLE_ROOT) { $env:ANGLE_ROOT = (Join-Path $RepoRoot "..\angle") }
if (-not $env:FLATBUFFERS_PATH) { $env:FLATBUFFERS_PATH = (Join-Path $RepoRoot "external\flatbuffers") }

Write-Host "ENABLE_GFXSTREAM_ANGLE=1"
Write-Host "AEMU_COMMON_PATH=$env:AEMU_COMMON_PATH"
Write-Host "ANGLE_ROOT=$env:ANGLE_ROOT"
Write-Host "FLATBUFFERS_PATH=$env:FLATBUFFERS_PATH"

& (Join-Path $RepoRoot "build_all.bat")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`nAfter build, run:"
Write-Host "  .\scripts\run_chromeos_firmware_gfx.ps1 -TimeoutSecs 180"
