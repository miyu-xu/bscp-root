# OVMF + disk with gfxstream display (requires crosvm built with gfxstream feature).
# Run build_all.bat with ENABLE_GFXSTREAM_ANGLE=1 first, or set GFXSTREAM_PATH.
param(
    [int]$TimeoutSecs = 180,
    [string]$LogDir = ""
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$BinDir = Join-Path $RepoRoot "out\dist\windows\bin"
$Crosvm = Join-Path $BinDir "crosvm.exe"

$ErrProbe = Join-Path $env:TEMP "crosvm-gfx-help.txt"
cmd /c "`"$Crosvm`" run --help 2>&1" | Out-File -FilePath $ErrProbe -Encoding utf8
$help = Get-Content $ErrProbe -Raw -ErrorAction SilentlyContinue
$dll = Join-Path $BinDir "gfxstream_backend.dll"
if (-not (Test-Path $dll)) {
    Write-Warning @"
gfxstream_backend.dll not in out\dist\windows\bin.
Rebuild: .\scripts\build_chromeos_firmware_gfx.ps1
  (sets ENABLE_GFXSTREAM_ANGLE=1, uses hardware\google\aemu)
"@
}
if ($help -match "unknown variant.*gfxstream|expected.*2D") {
    Write-Warning "crosvm.exe lacks gfxstream feature — run build_chromeos_firmware_gfx.ps1 first."
}

& (Join-Path $RepoRoot "scripts\run_crosvm_chromeos_firmware.ps1") `
    -Firmware ovmf -TimeoutSecs $TimeoutSecs -LogDir $LogDir -UseGpu
