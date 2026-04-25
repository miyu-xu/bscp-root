param(
    [ValidateSet("connect", "start-microdroid", "help")]
    [string]$Command = "help",
    [string]$RepoRoot = "",
    [int]$Cid = 0,
    [int]$AdbPort = 8000,
    [int]$GuestAdbPort = 5555,
    [string]$AdbPath = "adb",
    [string]$LogDir = "",
    [string]$WorkDir = "",
    [string]$ServiceRoot = "",
    [string]$DebugPolicyJson = "",
    [switch]$AutoConnect,
    [switch]$PersistVirtmgr,
    [switch]$NoShell,
    [switch]$NoRoot,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$VmArgs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
if (Get-Variable -Name PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue) {
    $PSNativeCommandUseErrorActionPreference = $false
}

function Require-Path {
    param(
        [string]$Path,
        [string]$Label
    )

    if (-not (Test-Path -LiteralPath $Path)) {
        throw "$Label not found: $Path"
    }
}

function Get-RepoRoot {
    if ($RepoRoot) {
        return $RepoRoot
    }
    return (Split-Path -Parent (Split-Path -Parent $PSCommandPath))
}

function Get-LatestVmLogDir {
    param([string]$LogsRoot)

    $dirs = Get-ChildItem -Path $LogsRoot -Directory -ErrorAction SilentlyContinue |
        Where-Object {
            $_.Name -like "windows-run-microdroid-*" -or
            $_.Name -like "windows-run-app-*" -or
            $_.Name -like "windows-run-raw-*" -or
            $_.Name -like "windows-vm-shell-*"
        } |
        Sort-Object LastWriteTime -Descending
    return $dirs | Select-Object -First 1
}

function Resolve-CidFromTrace {
    param(
        [string]$TracePath,
        [long]$StartOffset = 0
    )

    if (-not (Test-Path -LiteralPath $TracePath)) {
        return $null
    }

    $content = [System.IO.File]::ReadAllText($TracePath)
    if ($StartOffset -gt 0 -and $content.Length -gt $StartOffset) {
        $content = $content.Substring($StartOffset)
    }
    $match = [regex]::Match($content, 'cid=(\d+)')
    if ($match) {
        return [int]$match.Groups[1].Value
    }

    return $null
}

function Resolve-DefaultDebugPolicy {
    param([string]$ResolvedRepoRoot)

    if ($DebugPolicyJson) {
        return $DebugPolicyJson
    }

    $defaultPath = Join-Path $ResolvedRepoRoot "packages\modules\Virtualization\android\virtmgr\examples\windows_debug_policy.json"
    if (Test-Path -LiteralPath $defaultPath) {
        return $defaultPath
    }

    return ""
}

function Wait-ForTracePattern {
    param(
        [string]$TracePath,
        [string]$Pattern,
        [long]$StartOffset = 0,
        [int]$TimeoutSeconds = 60
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        if (Test-Path -LiteralPath $TracePath) {
            $content = [System.IO.File]::ReadAllText($TracePath)
            if ($StartOffset -gt 0 -and $content.Length -gt $StartOffset) {
                $content = $content.Substring($StartOffset)
            }
            if ($content -match $Pattern) {
                return $true
            }
        }
        Start-Sleep -Milliseconds 500
    }
    return $false
}

function Get-VirtmgrServiceRoot {
    param([string]$ResolvedRepoRoot)

    if ($ServiceRoot) {
        return $ServiceRoot
    }
    return Join-Path (Join-Path $ResolvedRepoRoot "out\dist") "logs\windows-virtmgr-service"
}

function Get-VirtmgrServiceTracePath {
    param([string]$ResolvedRepoRoot)

    $resolvedServiceRoot = Get-VirtmgrServiceRoot -ResolvedRepoRoot $ResolvedRepoRoot
    return Join-Path $resolvedServiceRoot "virtmgr-trace.log"
}

function Write-AdbLog {
    param(
        [string]$LogPath,
        [string]$Message
    )

    if (-not $LogPath) {
        return
    }

    $line = "[{0}] {1}" -f (Get-Date -Format "yyyy-MM-dd HH:mm:ss"), $Message
    Add-Content -Path $LogPath -Value $line -Encoding utf8
}

function Invoke-NativeCapture {
    param(
        [string]$FilePath,
        [string[]]$ArgumentList
    )

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $FilePath
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true
    $quotedArgs = foreach ($arg in $ArgumentList) {
        if ($arg -match '[\s"]') {
            '"' + ($arg -replace '"', '\"') + '"'
        } else {
            $arg
        }
    }
    $psi.Arguments = ($quotedArgs -join ' ')

    $proc = [System.Diagnostics.Process]::Start($psi)
    $stdout = $proc.StandardOutput.ReadToEnd()
    $stderr = $proc.StandardError.ReadToEnd()
    $proc.WaitForExit()

    return [pscustomobject]@{
        ExitCode = $proc.ExitCode
        Output = ($stdout + $stderr).Trim()
    }
}

function Wait-ForAdbDeviceState {
    param(
        [string]$ResolvedAdb,
        [string]$Serial,
        [string]$DesiredState = "device",
        [int]$Attempts = 30,
        [int]$DelaySeconds = 1,
        [string]$AdbLogPath = ""
    )

    for ($attempt = 1; $attempt -le $Attempts; $attempt++) {
        $result = Invoke-NativeCapture -FilePath $ResolvedAdb -ArgumentList @("-s", $Serial, "get-state")
        Write-AdbLog -LogPath $AdbLogPath -Message ("adb -s {0} get-state [attempt {1}/{2}] => {3}" -f $Serial, $attempt, $Attempts, $result.Output)
        if (($result.ExitCode -eq 0) -and ($result.Output -eq $DesiredState)) {
            return $true
        }
        Start-Sleep -Seconds $DelaySeconds
    }

    return $false
}

function Get-AdbFailureHint {
    param(
        [string]$ResolvedRepoRoot,
        [string]$FailureContextDir
    )

    if ($FailureContextDir) {
        $guestLogs = Get-ChildItem -Path $FailureContextDir -Recurse -File -Filter "guest-virtio-console*.txt" -ErrorAction SilentlyContinue
        foreach ($guestLog in $guestLogs) {
            if (Select-String -Path $guestLog.FullName -Pattern "adbd listening on vsock:5555" -Quiet) {
                return "guest adbd is already running on vsock:5555, so inspect adb-connect.log, virtmgr-trace.log, and temp\\virtmgr\\<cid>\\crosvm-stderr.txt in the same run directory to see whether the built-in Windows host bridge started and whether the guest accepted the host-initiated connection."
            }
            if (Select-String -Path $guestLog.FullName -Pattern "service adbd not found" -Quiet) {
                return "guest init triggered ADB enablement, but the current Microdroid runtime does not include the adbd service ('service adbd not found'). The current out\\dist\\com.android.virt payload is missing adbd, so adb connect cannot succeed."
            }
        }
    }

    $runtimeRoot = Join-Path $ResolvedRepoRoot "out\dist"
    if (Test-Path -LiteralPath $runtimeRoot) {
        $adbdArtifacts = Get-ChildItem -Path $runtimeRoot -Recurse -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -like "com.android.adbd*" -or $_.Name -eq "adbd" } |
            Select-Object -First 1
        if (-not $adbdArtifacts) {
            return "the current out\\dist runtime does not contain com.android.adbd artifacts, so the guest never exposes vsock:5555 for ADB."
        }
    }

    return "see adb-connect.log, vm-run-microdroid.log, and temp\\virtmgr\\<cid>\\guest-virtio-console*.txt in the same run directory."
}

function Invoke-AdbConnect {
    param(
        [string]$ResolvedAdb,
        [int]$ResolvedPort,
        [string]$ResolvedRepoRoot,
        [string]$FailureContextDir,
        [string]$AdbLogPath,
        [switch]$SkipShell,
        [switch]$SkipRoot
    )

    $serial = "localhost:$ResolvedPort"
    $disconnectResult = Invoke-NativeCapture -FilePath $ResolvedAdb -ArgumentList @("disconnect", $serial)
    Write-AdbLog -LogPath $AdbLogPath -Message ("adb disconnect {0} => {1}" -f $serial, $disconnectResult.Output)

    $connected = $false
    for ($attempt = 1; $attempt -le 20; $attempt++) {
        $result = Invoke-NativeCapture -FilePath $ResolvedAdb -ArgumentList @("connect", $serial)
        Write-AdbLog -LogPath $AdbLogPath -Message ("adb connect {0} [attempt {1}/20] => {2}" -f $serial, $attempt, $result.Output)
        if (($result.ExitCode -eq 0) -and ($result.Output -match "connected to|already connected to")) {
            $connected = $true
            break
        }
        Start-Sleep -Seconds 1
    }

    if (-not $connected) {
        $hint = Get-AdbFailureHint -ResolvedRepoRoot $ResolvedRepoRoot -FailureContextDir $FailureContextDir
        throw "adb connect to $serial did not succeed; $hint"
    }

    if (-not (Wait-ForAdbDeviceState -ResolvedAdb $ResolvedAdb -Serial $serial -DesiredState "device" -Attempts 15 -AdbLogPath $AdbLogPath)) {
        $hint = Get-AdbFailureHint -ResolvedRepoRoot $ResolvedRepoRoot -FailureContextDir $FailureContextDir
        throw "adb transport for $serial did not become online; $hint"
    }

    if (-not $SkipRoot) {
        $rootResult = Invoke-NativeCapture -FilePath $ResolvedAdb -ArgumentList @("-s", $serial, "root")
        Write-AdbLog -LogPath $AdbLogPath -Message ("adb -s {0} root => {1}" -f $serial, $rootResult.Output)
        if (-not (Wait-ForAdbDeviceState -ResolvedAdb $ResolvedAdb -Serial $serial -DesiredState "device" -Attempts 15 -AdbLogPath $AdbLogPath)) {
            $hint = Get-AdbFailureHint -ResolvedRepoRoot $ResolvedRepoRoot -FailureContextDir $FailureContextDir
            throw "adb root for $serial did not return to online state; $hint"
        }
    }

    if (-not $SkipShell) {
        Write-AdbLog -LogPath $AdbLogPath -Message ("adb -s {0} shell" -f $serial)
        & $ResolvedAdb -s $serial shell
    }
}

function Start-MicrodroidWithAutoConnect {
    param(
        [string]$ResolvedRepoRoot,
        [string]$ResolvedAdb,
        [string]$ResolvedDebugPolicy
    )

    $vmWrapper = Join-Path $ResolvedRepoRoot "scripts\vm_windows.ps1"
    Require-Path -Path $vmWrapper -Label "vm_windows.ps1"

    if (-not $LogDir) {
        $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
        $script:LogDir = Join-Path (Join-Path $ResolvedRepoRoot "out\dist\logs") ("windows-vm-shell-{0}" -f $timestamp)
    }
    if (-not $WorkDir) {
        $script:WorkDir = Join-Path $LogDir "work"
    }

    New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
    $tracePath = if ($PersistVirtmgr) {
        Get-VirtmgrServiceTracePath -ResolvedRepoRoot $ResolvedRepoRoot
    } else {
        Join-Path $LogDir "virtmgr-trace.log"
    }
    $traceStartOffset = if (Test-Path -LiteralPath $tracePath) {
        ([System.IO.File]::ReadAllText($tracePath)).Length
    } else {
        0
    }

    $pwsh = (Get-Command powershell -ErrorAction SilentlyContinue).Source
    if (-not $pwsh) {
        $pwsh = (Get-Command pwsh -ErrorAction Stop).Source
    }

    $args = @(
        "-NoProfile",
        "-ExecutionPolicy", "Bypass",
        "-File", $vmWrapper,
        "-Command", "run-microdroid",
        "-RepoRoot", $ResolvedRepoRoot,
        "-LogDir", $LogDir,
        "-WorkDir", $WorkDir,
        "-KeepTemp",
        "-CaptureGuestConsole",
        "-CaptureCrosvmStdio"
    )
    if ($PersistVirtmgr) {
        $args += "-PersistVirtmgr"
        $args += @("-ServiceRoot", (Get-VirtmgrServiceRoot -ResolvedRepoRoot $ResolvedRepoRoot))
    }

    if ($ResolvedDebugPolicy) {
        $args += @("-DebugPolicyJson", $ResolvedDebugPolicy)
    }
    $args += @("--adb-tcp-port", $AdbPort)
    if ($VmArgs) {
        $args += $VmArgs
    }

    $proc = Start-Process -FilePath $pwsh -ArgumentList $args -PassThru

    if (-not (Wait-ForTracePattern -TracePath $tracePath -Pattern "notifyPayloadReady" -StartOffset $traceStartOffset -TimeoutSeconds 90)) {
        throw "Timed out waiting for notifyPayloadReady. Check $LogDir"
    }

    $adbLog = Join-Path $LogDir "adb-connect.log"
    $resolvedCid = Resolve-CidFromTrace -TracePath $tracePath -StartOffset $traceStartOffset
    if (-not $resolvedCid) {
        $resolvedCid = 2048
    }

    Write-Host "VM process          : $($proc.Id)"
    Write-Host "ADB tcp bridge      : localhost:$AdbPort (hosted by virtmgr)"
    Write-Host "CID                 : $resolvedCid"
    Write-Host "LogDir              : $LogDir"

    Invoke-AdbConnect -ResolvedAdb $ResolvedAdb -ResolvedPort $AdbPort -ResolvedRepoRoot $ResolvedRepoRoot -FailureContextDir $LogDir -AdbLogPath $adbLog -SkipShell:$NoShell -SkipRoot:$NoRoot
}

function Connect-Microdroid {
    param(
        [string]$ResolvedRepoRoot,
        [string]$ResolvedAdb
    )

    $resolvedCid = $Cid
    $resolvedLogDir = $LogDir

    if (-not $resolvedLogDir) {
        $latest = Get-LatestVmLogDir -LogsRoot (Join-Path $ResolvedRepoRoot "out\dist\logs")
        if ($latest) {
            $resolvedLogDir = $latest.FullName
        }
    }

    if (-not $resolvedCid) {
        if ($resolvedLogDir) {
            $tracePath = if ($PersistVirtmgr) {
                Get-VirtmgrServiceTracePath -ResolvedRepoRoot $ResolvedRepoRoot
            } else {
                Join-Path $resolvedLogDir "virtmgr-trace.log"
            }
            $resolvedCid = Resolve-CidFromTrace -TracePath $tracePath
        }
        if (-not $resolvedCid) {
            $resolvedCid = 2048
        }
    }

    if (-not $resolvedLogDir) {
        $resolvedLogDir = Join-Path (Join-Path $ResolvedRepoRoot "out\dist\logs") "windows-vm-shell-connect"
    }

    New-Item -ItemType Directory -Force -Path $resolvedLogDir | Out-Null
    $adbLog = Join-Path $resolvedLogDir "adb-connect.log"
    Write-Host "ADB tcp bridge      : localhost:$AdbPort"
    Write-Host "CID                 : $resolvedCid"
    Write-Host "LogDir              : $resolvedLogDir"

    Invoke-AdbConnect -ResolvedAdb $ResolvedAdb -ResolvedPort $AdbPort -ResolvedRepoRoot $ResolvedRepoRoot -FailureContextDir $resolvedLogDir -AdbLogPath $adbLog -SkipShell:$NoShell -SkipRoot:$NoRoot
}

$resolvedRepoRoot = Get-RepoRoot
$vmWrapper = Join-Path $resolvedRepoRoot "scripts\vm_windows.ps1"
$resolvedDebugPolicy = Resolve-DefaultDebugPolicy -ResolvedRepoRoot $resolvedRepoRoot
$resolvedAdb = (Get-Command $AdbPath -ErrorAction Stop).Source

switch ($Command) {
    "help" {
        @"
vm_shell_windows.ps1 provides Windows helpers roughly equivalent to packages\modules\Virtualization\android\vm\vm_shell.sh

Available commands:
  start-microdroid [-AutoConnect] [-- extra_args]
    Starts Microdroid through scripts\vm_windows.ps1.
    With -AutoConnect, this script enables the Windows debug policy, waits for payload ready,
    asks virtmgr to host a localhost TCP -> guest vsock:5555 bridge, and then runs adb connect/get-state/root/shell.
    Add -PersistVirtmgr to keep one Windows virtmgr service alive across commands so vm list / vm console can reconnect to the same service state.

  connect [-Cid <cid>]
    Reconnects adb to localhost:<AdbPort>.
    This expects the VM to already be running with the built-in Windows host ADB bridge enabled.
"@ | Write-Host
    }
    "connect" {
        Connect-Microdroid -ResolvedRepoRoot $resolvedRepoRoot -ResolvedAdb $resolvedAdb
    }
    "start-microdroid" {
            if ($AutoConnect) {
                Start-MicrodroidWithAutoConnect -ResolvedRepoRoot $resolvedRepoRoot -ResolvedAdb $resolvedAdb -ResolvedDebugPolicy $resolvedDebugPolicy
            } else {
                $args = @(
                    "-Command", "run-microdroid",
                    "-RepoRoot", $resolvedRepoRoot
                )
                if ($PersistVirtmgr) {
                    $args += "-PersistVirtmgr"
                    $args += @("-ServiceRoot", (Get-VirtmgrServiceRoot -ResolvedRepoRoot $resolvedRepoRoot))
                }
                if ($resolvedDebugPolicy) {
                    $args += @("-DebugPolicyJson", $resolvedDebugPolicy)
                }
            if ($LogDir) {
                $args += @("-LogDir", $LogDir)
            }
            if ($WorkDir) {
                $args += @("-WorkDir", $WorkDir)
            }
            if ($VmArgs) {
                $args += $VmArgs
            }
            & $vmWrapper @args
            exit $LASTEXITCODE
        }
    }
}
