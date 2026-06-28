param(
    [string]$LogDir = ""
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
if (-not $LogDir) {
    $LogDir = Join-Path $RepoRoot "out\dist\logs\android-windows"
}

Write-Host "Windows direct-runner parity validation"
Write-Host "Log dir: $LogDir"

& "$PSScriptRoot\run_android_windows_gfxstream_angle.ps1" -DryRun -NoNetwork | Out-Null
Write-Host "[PASS] run_android_windows_gfxstream_angle.ps1 dry-run"

if (Test-Path $LogDir) {
    if (Test-Path "$PSScriptRoot\check_android_windows_markers.ps1") {
        & "$PSScriptRoot\check_android_windows_markers.ps1" -LogDir $LogDir
        Write-Host "[PASS] boot completion markers"
    }
    if (Test-Path "$PSScriptRoot\check_android_windows_gfx_markers.ps1") {
        & "$PSScriptRoot\check_android_windows_gfx_markers.ps1" -LogDir $LogDir
        Write-Host "[PASS] gfxstream markers"
    }
    & "$PSScriptRoot\check_android_windows_parity_markers.ps1" -LogDir $LogDir
    Write-Host "[PASS] radio-adjacent parity markers"
} else {
    Write-Host "[SKIP] no captured log dir at $LogDir"
    Write-Host "       Run: scripts\run_android_windows_gfxstream_angle.ps1 -FullHvc -TimeoutSecs 420"
}

Write-Host ""
Write-Host "Rebuild crosvm with net,slirp before full validation:"
Write-Host "  build_all.bat ENABLE_GFXSTREAM_ANGLE=1"
