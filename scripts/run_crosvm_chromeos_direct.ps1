# Minimal crosvm direct launch for ChromiumOS mmio@0 debugging (bypasses virtmgr).
param(
    [string]$LogDir = "$PSScriptRoot\..\out\dist\logs\cros-direct",
    [int]$TimeoutSecs = 40,
    [ValidateSet("minimal", "vboot")]
    [string]$CmdlineProfile = "minimal"
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$BinDir = Join-Path $RepoRoot "out\dist\windows\bin"
$Img = Join-Path $RepoRoot "out\dist\img\amd64-generic_test_image.bin"
$Kernel = Join-Path $RepoRoot "out\dist\img\amd64-generic_kernel.bin"
$Crosvm = Join-Path $BinDir "crosvm.exe"

New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$SerialLog = Join-Path $LogDir "guest-serial-num1.txt"
$HvcLog = Join-Path $LogDir "guest-virtio-console1.txt"
$CrosvmErr = Join-Path $LogDir "crosvm-stderr.txt"
$CrosvmOut = Join-Path $LogDir "crosvm-stdout.txt"

if (-not (Test-Path $Crosvm)) { throw "crosvm not found: $Crosvm" }
if (-not (Test-Path $Img)) { throw "image not found: $Img" }
if (-not (Test-Path $Kernel)) { throw "kernel not found: $Kernel" }

$paramsMinimal = "console=ttyS0,115200n8 earlyprintk=serial,ttyS0,8250,115200n8 loglevel=7 init=/sbin/init root=/dev/vda3 rw"
$paramsVboot = Get-Content (Join-Path $RepoRoot "out\dist\img\amd64-generic.cmdline.resolved.txt") -Raw
$params = if ($CmdlineProfile -eq "vboot") { $paramsVboot.Trim() } else { $paramsMinimal }

$env:PATH = "C:\workspace\bscp\angle\out\Release-GfxAngle-Clang;$BinDir;$env:PATH"

$crosvmArgs = @(
    "--log-level", "info,disk=warn",
    "run",
    "--disable-sandbox",
    "--cid", "4096",
    "--mem", "4096",
    "--cpus", "1",
    "--no-usb",
    "--serial", "type=file,path=$SerialLog,hardware=serial,num=1,earlycon=true",
    "--serial", "type=sink,hardware=serial,num=2",
    "--serial", "type=file,path=$HvcLog,hardware=virtio-console,num=1,console=true",
    "--serial", "type=sink,hardware=virtio-console,num=2",
    "--params", $params,
    "--block", "path=$Img,ro=true,lock=false,sparse=false",
    $Kernel
)

Write-Host "LogDir: $LogDir"
Write-Host "Profile: $CmdlineProfile"
Write-Host "Params: $($params.Substring(0, [Math]::Min(120, $params.Length)))..."
Write-Host "Start: $(Get-Date -Format 'HH:mm:ss')"

$proc = Start-Process -FilePath $Crosvm -ArgumentList ($crosvmArgs | ForEach-Object {
        if ($_ -match '[\s"]') { '"{0}"' -f ($_.Replace('"', '\"')) } else { $_ }
    }) `
    -RedirectStandardError $CrosvmErr -RedirectStandardOutput $CrosvmOut `
    -PassThru -NoNewWindow

$deadline = (Get-Date).AddSeconds($TimeoutSecs)
while (-not $proc.HasExited -and (Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 2
}
if (-not $proc.HasExited) {
    Write-Host "Timeout after ${TimeoutSecs}s, stopping crosvm pid=$($proc.Id)"
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep 1
}

Write-Host "End: $(Get-Date -Format 'HH:mm:ss') exit=$($proc.ExitCode)"
Write-Host ""
foreach ($f in @($SerialLog, $HvcLog, $CrosvmErr)) {
    if (Test-Path $f) {
        $sz = (Get-Item $f).Length
        Write-Host "=== $([IO.Path]::GetFileName($f)) ($sz bytes) ==="
        if ($sz -gt 0 -and $sz -lt 8000) {
            Get-Content $f -TotalCount 40
        } elseif ($sz -gt 0) {
            Get-Content $f -TotalCount 25
            Write-Host "... (truncated)"
        }
        Write-Host ""
    }
}
