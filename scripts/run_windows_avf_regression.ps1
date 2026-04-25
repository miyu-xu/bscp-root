param(
    [string]$RepoRoot = "",
    [string]$DistRoot = "",
    [string]$OutputRoot = "",
    [int]$AdbPort = 8035,
    [switch]$IncludeRunApp,
    [switch]$IncludeAdbScenario
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
if (Get-Variable -Name PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue) {
    $PSNativeCommandUseErrorActionPreference = $false
}

function Get-RepoRoot {
    if ($RepoRoot) {
        return $RepoRoot
    }
    return (Split-Path -Parent (Split-Path -Parent $PSCommandPath))
}

function Invoke-Step {
    param(
        [string]$Label,
        [string]$FilePath,
        [string[]]$Arguments,
        [int[]]$AllowExitCodes = @(0)
    )

    Write-Host "=== $Label ==="
    & powershell -NoProfile -ExecutionPolicy Bypass -File $FilePath @Arguments
    if ($AllowExitCodes -notcontains $LASTEXITCODE) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
}

function Resolve-CidFromTrace {
    param([string]$TracePath)

    if (-not (Test-Path -LiteralPath $TracePath)) {
        throw "Trace file not found: $TracePath"
    }
    $match = Select-String -Path $TracePath -Pattern 'cid=(\d+)' | Select-Object -Last 1
    if (-not $match) {
        throw "Could not resolve CID from $TracePath"
    }
    return [int]$match.Matches[0].Groups[1].Value
}

function Wait-ForTracePattern {
    param(
        [string]$TracePath,
        [string]$Pattern,
        [int]$TimeoutSeconds = 30
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if ((Test-Path -LiteralPath $TracePath) -and (Select-String -Path $TracePath -Pattern $Pattern -Quiet)) {
            return
        }
        Start-Sleep -Milliseconds 500
    }

    throw "Timed out waiting for pattern '$Pattern' in $TracePath"
}

function Stop-AvfProcesses {
    $processes = Get-Process -ErrorAction SilentlyContinue |
        Where-Object { $_.ProcessName -in @("vm", "virtmgr", "crosvm") }
    foreach ($process in $processes) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    }
}

$resolvedRepoRoot = Get-RepoRoot
if (-not $DistRoot) {
    $DistRoot = Join-Path $resolvedRepoRoot "out\dist"
}
if (-not $OutputRoot) {
    $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $OutputRoot = Join-Path $DistRoot ("logs\windows-regression-{0}" -f $timestamp)
}

$vmWrapper = Join-Path $resolvedRepoRoot "scripts\vm_windows.ps1"
$vmShell = Join-Path $resolvedRepoRoot "scripts\vm_shell_windows.ps1"
$checkMarkers = Join-Path $resolvedRepoRoot "scripts\check_windows_avf_markers.ps1"
$serviceRoot = Join-Path $OutputRoot "service"
$serviceTrace = Join-Path $serviceRoot "virtmgr-trace.log"

New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null
Stop-AvfProcesses

Invoke-Step -Label "validate-prereqs" -FilePath $vmWrapper -Arguments @(
    "-Command", "validate-prereqs",
    "-RepoRoot", $resolvedRepoRoot,
    "-DistRoot", $DistRoot,
    "-LogDir", (Join-Path $OutputRoot "validate-prereqs")
)

Invoke-Step -Label "info" -FilePath $vmWrapper -Arguments @(
    "-Command", "info",
    "-RepoRoot", $resolvedRepoRoot,
    "-DistRoot", $DistRoot,
    "-LogDir", (Join-Path $OutputRoot "info")
)

Invoke-Step -Label "create-partition" -FilePath $vmWrapper -Arguments @(
    "-Command", "create-partition",
    "-RepoRoot", $resolvedRepoRoot,
    "-DistRoot", $DistRoot,
    "-LogDir", (Join-Path $OutputRoot "create-partition"),
    "-PartitionPath", (Join-Path $OutputRoot "create-partition\writable.img"),
    "-PartitionSize", "1048576"
)

Invoke-Step -Label "create-idsig" -FilePath $vmWrapper -Arguments @(
    "-Command", "create-idsig",
    "-RepoRoot", $resolvedRepoRoot,
    "-DistRoot", $DistRoot,
    "-LogDir", (Join-Path $OutputRoot "create-idsig"),
    "-OutputPath", (Join-Path $OutputRoot "create-idsig\app.idsig")
)

$runMicrodroidLog = Join-Path $OutputRoot "run-microdroid"
Invoke-Step -Label "run-microdroid" -FilePath $vmWrapper -Arguments @(
    "-Command", "run-microdroid",
    "-RepoRoot", $resolvedRepoRoot,
    "-DistRoot", $DistRoot,
    "-LogDir", $runMicrodroidLog,
    "-KeepTemp",
    "-CaptureGuestConsole",
    "-CaptureCrosvmStdio",
    "-TimeoutSecs", "120"
) -AllowExitCodes @(0, 124)
Invoke-Step -Label "check run-microdroid markers" -FilePath $checkMarkers -Arguments @(
    "-Scenario", "run-microdroid",
    "-LogDir", $runMicrodroidLog
)
Stop-AvfProcesses

if ($IncludeRunApp) {
    $runAppLog = Join-Path $OutputRoot "run-app"
    Invoke-Step -Label "run-app" -FilePath $vmWrapper -Arguments @(
        "-Command", "run-app",
        "-RepoRoot", $resolvedRepoRoot,
        "-DistRoot", $DistRoot,
        "-LogDir", $runAppLog,
        "-KeepTemp",
        "-CaptureGuestConsole",
        "-CaptureCrosvmStdio",
        "-TimeoutSecs", "120"
    ) -AllowExitCodes @(0, 124)
    Invoke-Step -Label "check run-app markers" -FilePath $checkMarkers -Arguments @(
        "-Scenario", "run-app",
        "-LogDir", $runAppLog
    )
    Stop-AvfProcesses
}

Invoke-Step -Label "stop stale service" -FilePath $vmWrapper -Arguments @(
    "-Command", "stop-service",
    "-RepoRoot", $resolvedRepoRoot,
    "-DistRoot", $DistRoot,
    "-ServiceRoot", $serviceRoot
)

$persistentRunLog = Join-Path $OutputRoot "persistent-run-microdroid"
Invoke-Step -Label "persistent run-microdroid" -FilePath $vmWrapper -Arguments @(
    "-Command", "run-microdroid",
    "-RepoRoot", $resolvedRepoRoot,
    "-DistRoot", $DistRoot,
    "-LogDir", $persistentRunLog,
    "-KeepTemp",
    "-CaptureGuestConsole",
    "-CaptureCrosvmStdio",
    "-PersistVirtmgr",
    "-ServiceRoot", $serviceRoot
)
Wait-ForTracePattern -TracePath $serviceTrace -Pattern "notifyPayloadReady" -TimeoutSeconds 30
Invoke-Step -Label "check persistent run markers" -FilePath $checkMarkers -Arguments @(
    "-Scenario", "run-microdroid",
    "-LogDir", $persistentRunLog,
    "-TracePath", $serviceTrace
)

$cid = Resolve-CidFromTrace -TracePath $serviceTrace

$listLogDir = Join-Path $OutputRoot "list"
Invoke-Step -Label "vm list" -FilePath $vmWrapper -Arguments @(
    "-Command", "list",
    "-RepoRoot", $resolvedRepoRoot,
    "-DistRoot", $DistRoot,
    "-LogDir", $listLogDir,
    "-PersistVirtmgr",
    "-ServiceRoot", $serviceRoot
)
Invoke-Step -Label "check list markers" -FilePath $checkMarkers -Arguments @(
    "-Scenario", "list",
    "-LogDir", $listLogDir,
    "-ExpectedCid", $cid
)

$consoleLogDir = Join-Path $OutputRoot "console"
Invoke-Step -Label "vm console" -FilePath $vmWrapper -Arguments @(
    "-Command", "console",
    "-RepoRoot", $resolvedRepoRoot,
    "-DistRoot", $DistRoot,
    "-LogDir", $consoleLogDir,
    "-PersistVirtmgr",
    "-ServiceRoot", $serviceRoot,
    "-Cid", $cid,
    "--read-only",
    "--timeout-secs", "3"
)
Invoke-Step -Label "check console markers" -FilePath $checkMarkers -Arguments @(
    "-Scenario", "console",
    "-LogDir", $consoleLogDir,
    "-ExpectedCid", $cid
)

if ($IncludeAdbScenario) {
    $adbLogDir = Join-Path $OutputRoot "start-microdroid-adb"
    Invoke-Step -Label "start-microdroid -AutoConnect" -FilePath $vmShell -Arguments @(
        "-Command", "start-microdroid",
        "-RepoRoot", $resolvedRepoRoot,
        "-LogDir", $adbLogDir,
        "-PersistVirtmgr",
        "-ServiceRoot", $serviceRoot,
        "-AutoConnect",
        "-NoShell",
        "-NoRoot",
        "-AdbPort", $AdbPort
    )
    Invoke-Step -Label "check adb markers" -FilePath $checkMarkers -Arguments @(
        "-Scenario", "start-microdroid-adb",
        "-LogDir", $adbLogDir,
        "-TracePath", $serviceTrace
    )
}

Invoke-Step -Label "service-status" -FilePath $vmWrapper -Arguments @(
    "-Command", "service-status",
    "-RepoRoot", $resolvedRepoRoot,
    "-DistRoot", $DistRoot,
    "-ServiceRoot", $serviceRoot
)

Invoke-Step -Label "stop-service" -FilePath $vmWrapper -Arguments @(
    "-Command", "stop-service",
    "-RepoRoot", $resolvedRepoRoot,
    "-DistRoot", $DistRoot,
    "-ServiceRoot", $serviceRoot
)
Stop-AvfProcesses

if ($IncludeAdbScenario) {
    Write-Host "Windows AVF regression completed successfully, including optional ADB coverage."
} else {
    Write-Host "Windows AVF regression completed successfully."
}
Write-Host "Artifacts written to: $OutputRoot"
