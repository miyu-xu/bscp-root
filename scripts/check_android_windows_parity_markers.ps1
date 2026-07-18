param(
    [Parameter(Mandatory = $true)][string]$LogDir
)

$ErrorActionPreference = "Stop"

$Logcat = Join-Path $LogDir "logcat-hvc2.txt"
$KernelLog = Join-Path $LogDir "hvc.txt"
$CommandFile = Join-Path $LogDir "crosvm-command.txt"

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

function Get-EnabledConfig {
    param([string]$Name, [bool]$Default = $true)
    if (-not (Test-Path $CommandFile)) {
        return $Default
    }
    $line = Get-Content $CommandFile | Where-Object { $_ -like "$Name=*" } | Select-Object -First 1
    if (-not $line) {
        return $Default
    }
    return ($line.Substring($Name.Length + 1) -eq "True")
}

$BluetoothEnabled = Get-EnabledConfig -Name "BLUETOOTH_ENABLED"
$NfcEnabled = Get-EnabledConfig -Name "NFC_ENABLED"
$ModemEnabled = Get-EnabledConfig -Name "MODEM_ENABLED"

Require-Marker -Pattern "Finished executing PersistentDataBlockService.onStart" -File $Logcat `
    -Label "PersistentDataBlockService initialized"
Require-Marker -Pattern "OnBootPhase_1000" -File $Logcat -Label "system_server boot phase 1000"
Require-Marker -Pattern "processing action (sys.boot_completed=1)" -File $KernelLog -Label "sys.boot_completed=1"

Reject-Marker -Pattern "Service PersistentDataBlockService init timeout" -File $Logcat `
    -Label "PersistentDataBlockService timeout"
Reject-Marker -Pattern "FATAL EXCEPTION IN SYSTEM PROCESS" -File $Logcat -Label "system_server fatal exception"
Reject-Marker -Pattern "Exit zygote because system server" -File $Logcat -Label "system_server terminated"

Reject-Marker -Pattern "hdlc_interface.cpp:206: I/O error" -File $Logcat `
    -Label "ThreadNetwork HDLC I/O error"
if ($BluetoothEnabled) {
    Reject-Marker -Pattern "Can't start stack, last instance: starting HciHal" -File $Logcat `
        -Label "Bluetooth HciHal startup failure"
}
if ($NfcEnabled) {
    Reject-Marker -Pattern "NFA_DM_NFCC_TIMEOUT_EVT; abort" -File $Logcat `
        -Label "NFC NFCC timeout"
}
if ($ModemEnabled) {
    Reject-Marker -Pattern "ANR in com.android.phone" -File $Logcat `
        -Label "com.android.phone ANR with host modem enabled"
    Reject-Marker -Pattern "Service MODEM has died" -File $Logcat `
        -Label "radio modem service death"
}

Write-Host "Marker check passed: android-windows parity"
