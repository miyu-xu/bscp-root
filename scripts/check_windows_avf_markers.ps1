param(
    [ValidateSet("run-microdroid", "run-app", "start-microdroid-adb", "list", "console")]
    [string]$Scenario,
    [string]$LogDir,
    [string]$TracePath = "",
    [int]$ExpectedCid = 0
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Require-File {
    param(
        [string]$Path,
        [string]$Label
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        throw "$Label not found: $Path"
    }
}

function Assert-FileContains {
    param(
        [string]$Path,
        [string]$Pattern,
        [string]$Label
    )

    Require-File -Path $Path -Label $Label
    if (-not (Select-String -Path $Path -Pattern $Pattern -Quiet)) {
        throw "$Label is missing pattern '$Pattern': $Path"
    }
}

function Get-GuestConsoleLog {
    param([string]$ResolvedLogDir)

    $guestLog = Get-ChildItem -Path $ResolvedLogDir -Recurse -File -Filter "guest-virtio-console3.txt" -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if ($guestLog) {
        return $guestLog.FullName
    }
    return $null
}

if (-not $TracePath) {
    $TracePath = Join-Path $LogDir "virtmgr-trace.log"
}

switch ($Scenario) {
    "run-microdroid" {
        Assert-FileContains -Path $TracePath -Pattern "notifyPayloadStarted" -Label "virtmgr trace"
        Assert-FileContains -Path $TracePath -Pattern "notifyPayloadReady" -Label "virtmgr trace"
    }
    "run-app" {
        Assert-FileContains -Path $TracePath -Pattern "notifyPayloadStarted" -Label "virtmgr trace"
        Assert-FileContains -Path $TracePath -Pattern "notifyPayloadReady" -Label "virtmgr trace"
    }
    "start-microdroid-adb" {
        Assert-FileContains -Path $TracePath -Pattern "notifyPayloadReady" -Label "virtmgr trace"
        $guestConsole = Get-GuestConsoleLog -ResolvedLogDir $LogDir
        if (-not $guestConsole) {
            throw "guest-virtio-console3.txt not found under $LogDir"
        }
        Assert-FileContains -Path $guestConsole -Pattern "adbd listening on vsock:5555" -Label "guest console"
        Assert-FileContains -Path (Join-Path $LogDir "adb-connect.log") -Pattern "adb connect localhost:\d+ .*connected to localhost" -Label "adb log"
        Assert-FileContains -Path (Join-Path $LogDir "adb-connect.log") -Pattern "get-state .*=> device" -Label "adb log"
    }
    "list" {
        $listLog = Join-Path $LogDir "vm-list.log"
        Assert-FileContains -Path $listLog -Pattern "Running VMs:" -Label "vm list log"
        if ($ExpectedCid -gt 0) {
            Assert-FileContains -Path $listLog -Pattern $ExpectedCid.ToString() -Label "vm list log"
        }
    }
    "console" {
        $consoleLog = Join-Path $LogDir "vm-console.log"
        Assert-FileContains -Path $consoleLog -Pattern "Connecting to Windows VM console for CID" -Label "vm console log"
        if ($ExpectedCid -gt 0) {
            Assert-FileContains -Path $consoleLog -Pattern $ExpectedCid.ToString() -Label "vm console log"
        }
    }
}

Write-Host "Marker check passed: $Scenario"
