param(
    [Parameter(Mandatory = $true)][string]$LogDir
)

$ErrorActionPreference = "Stop"

$Logcat = Join-Path $LogDir "logcat-hvc2.txt"
$KernelLog = Join-Path $LogDir "hvc.txt"

if (-not (Test-Path $Logcat) -or -not (Test-Path $KernelLog)) {
    throw "Missing Android logs under $LogDir"
}

function Require-Marker {
    param([string]$Pattern, [string]$File, [string]$Label)
    if (-not (Select-String -Path $File -Pattern $Pattern -SimpleMatch -Quiet)) {
        throw "Missing marker: $Label"
    }
}

function Reject-Marker {
    param([string]$Pattern, [string]$File, [string]$Label)
    if (Select-String -Path $File -Pattern $Pattern -SimpleMatch -Quiet) {
        throw "Unexpected marker: $Label"
    }
}

Require-Marker -Pattern "processing action (sys.boot_completed=1)" -File $KernelLog -Label "sys.boot_completed=1"
Reject-Marker -Pattern "FATAL EXCEPTION IN SYSTEM PROCESS" -File $Logcat -Label "system_server fatal exception"

Write-Host "Marker check passed: android-windows boot"
