param(
    [ValidateSet("validate-prereqs", "run-microdroid", "run-app", "run", "info", "list", "console", "check-feature-enabled", "create-partition", "create-idsig", "service-status", "stop-service")]
    [string]$Command = "run-microdroid",
    [string]$RepoRoot = "",
    [string]$DistRoot = "",
    [string]$WorkDir = "",
    [string]$LogDir = "",
    [string]$ServiceRoot = "",
    [string]$TempRoot = "",
    [string]$Apk = "",
    [string]$Idsig = "",
    [string]$Instance = "",
    [string]$PayloadBinaryName = "MicrodroidEmptyPayloadJniLib.so",
    [string]$Config = "",
    [string]$Feature = "dice_changes",
    [string]$PartitionPath = "",
    [UInt64]$PartitionSize = 0,
    [string]$PartitionType = "raw",
    [string]$OutputPath = "",
    [int]$Cid = 0,
    [string]$Name = "",
    [string]$Console = "",
    [string]$ConsoleIn = "",
    [string]$GuestLog = "",
    [string]$TraceFile = "",
    [string]$VmclientTraceFile = "",
    [string]$DebugPolicyJson = "",
    [switch]$Protected,
    [switch]$KeepTemp,
    [switch]$CaptureGuestConsole,
    [switch]$CaptureCrosvmStdio,
    [switch]$PersistVirtmgr,
    [switch]$SkipHypervisorCheck,
    [switch]$DryRun,
    [int]$TimeoutSecs = 0,
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

function New-ParentDirectory {
    param([string]$Path)

    $parent = Split-Path -Parent $Path
    if ($parent) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }
}

function Write-Utf8File {
    param(
        [string]$Path,
        [string]$Content
    )

    New-ParentDirectory -Path $Path
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

function Read-TextFileWithRetry {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return ""
    }

    for ($attempt = 0; $attempt -lt 20; $attempt++) {
        try {
            return [System.IO.File]::ReadAllText($Path)
        } catch {
            Start-Sleep -Milliseconds 250
        }
    }

    return ""
}

function Expand-CapexOriginalApex {
    param(
        [string]$CapexPath,
        [string]$TargetApexPath
    )

    $copyNeeded = -not (Test-Path -LiteralPath $TargetApexPath)
    if (-not $copyNeeded) {
        $sourceInfo = Get-Item -LiteralPath $CapexPath
        $targetInfo = Get-Item -LiteralPath $TargetApexPath
        $copyNeeded = $sourceInfo.LastWriteTimeUtc -gt $targetInfo.LastWriteTimeUtc
    }

    if (-not $copyNeeded) {
        return
    }

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [System.IO.Compression.ZipFile]::OpenRead($CapexPath)
    try {
        $entry = $archive.GetEntry("original_apex")
        if ($null -eq $entry) {
            throw "CAPEX does not contain original_apex: $CapexPath"
        }

        New-ParentDirectory -Path $TargetApexPath
        $entryStream = $entry.Open()
        try {
            $fileStream = [System.IO.File]::Open($TargetApexPath, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write, [System.IO.FileShare]::None)
            try {
                $entryStream.CopyTo($fileStream)
            } finally {
                $fileStream.Dispose()
            }
        } finally {
            $entryStream.Dispose()
        }
    } finally {
        $archive.Dispose()
    }
}

function Ensure-ApexInfoEntry {
    param(
        [xml]$Document,
        [string]$ModuleName,
        [string]$ModulePath,
        [string]$PreinstalledPath,
        [string]$VersionCode = "352090000"
    )

    $root = $Document.DocumentElement
    if (($null -eq $root) -or ($root.Name -ne "apex-info-list")) {
        throw "Invalid apex-info-list.xml: missing apex-info-list root"
    }

    $existing = @($root.SelectNodes("apex-info") | Where-Object { $_.moduleName -eq $ModuleName })
    if ($existing.Count -gt 0) {
        foreach ($entry in $existing) {
            $entry.SetAttribute("modulePath", $ModulePath)
            $entry.SetAttribute("preinstalledModulePath", $PreinstalledPath)
            $entry.SetAttribute("versionCode", $VersionCode)
            $entry.SetAttribute("lastUpdateMillis", [DateTimeOffset]::UtcNow.ToUnixTimeSeconds().ToString())
            $entry.SetAttribute("isFactory", "true")
            $entry.SetAttribute("isActive", "true")
            $entry.SetAttribute("provideSharedApexLibs", "false")
        }
        return
    }

    $node = $Document.CreateElement("apex-info")
    $node.SetAttribute("moduleName", $ModuleName)
    $node.SetAttribute("modulePath", $ModulePath)
    $node.SetAttribute("preinstalledModulePath", $PreinstalledPath)
    $node.SetAttribute("versionCode", $VersionCode)
    $node.SetAttribute("versionName", "")
    $node.SetAttribute("isFactory", "true")
    $node.SetAttribute("isActive", "true")
    $node.SetAttribute("lastUpdateMillis", [DateTimeOffset]::UtcNow.ToUnixTimeSeconds().ToString())
    $node.SetAttribute("provideSharedApexLibs", "false")
    [void]$root.AppendChild($node)
}

function Ensure-WindowsApexLayout {
    param([string]$ResolvedApexTreeRoot)

    $mountedApexRoot = Join-Path $ResolvedApexTreeRoot "apex"
    Require-Path -Path $mountedApexRoot -Label "mounted apex root"
    Require-Path -Path (Join-Path $mountedApexRoot "com.android.virt") -Label "mounted com.android.virt apex"
    $decompressedRoot = Join-Path $mountedApexRoot "decompressed"
    New-Item -ItemType Directory -Force -Path $decompressedRoot | Out-Null

    $apexInfoList = Join-Path $mountedApexRoot "apex-info-list.xml"
    if (Test-Path -LiteralPath $apexInfoList) {
        [xml]$xml = Get-Content -LiteralPath $apexInfoList -Raw
    } else {
        [xml]$xml = "<?xml version=""1.0"" encoding=""utf-8""?><apex-info-list />"
    }

    $root = $xml.DocumentElement
    if (($null -eq $root) -or ($root.Name -ne "apex-info-list")) {
        throw "Invalid apex-info-list.xml: missing apex-info-list root"
    }

    while ($root.HasChildNodes) {
        [void]$root.RemoveChild($root.FirstChild)
    }

    $apexSources = @(
        @{ Partition = "system"; Directory = Join-Path $ResolvedApexTreeRoot "system\apex" },
        @{ Partition = "system_ext"; Directory = Join-Path $ResolvedApexTreeRoot "system_ext\apex" }
    )

    foreach ($source in $apexSources) {
        if (-not (Test-Path -LiteralPath $source.Directory)) {
            continue
        }

        Get-ChildItem -LiteralPath $source.Directory -File |
            Where-Object { $_.Extension -in @(".apex", ".capex") } |
            Sort-Object Name |
            ForEach-Object {
                $moduleName = [System.IO.Path]::GetFileNameWithoutExtension($_.Name)
                $preinstalledPath = "/{0}/apex/{1}" -f $source.Partition, $_.Name
                if ($_.Extension -eq ".capex") {
                    $decompressedPath = Join-Path $decompressedRoot ($moduleName + ".apex")
                    Expand-CapexOriginalApex -CapexPath $_.FullName -TargetApexPath $decompressedPath
                    $modulePath = "/apex/decompressed/{0}.apex" -f $moduleName
                    $versionCode = "352090000"
                } else {
                    $modulePath = $preinstalledPath
                    $versionCode = "1"
                }

                Ensure-ApexInfoEntry `
                    -Document $xml `
                    -ModuleName $moduleName `
                    -ModulePath $modulePath `
                    -PreinstalledPath $preinstalledPath `
                    -VersionCode $versionCode
            }
    }

    $content = $xml.OuterXml -replace '^<\?xml[^>]+\?>', ''
    $content = '<?xml version="1.0" encoding="utf-8"?>' + $content
    Write-Utf8File -Path $apexInfoList -Content $content
}

function Get-HypervisorState {
    $featureNames = @("HypervisorPlatform", "VirtualMachinePlatform", "Microsoft-Hyper-V-All")
    $states = @{}

    foreach ($featureName in $featureNames) {
        try {
            $feature = Get-WindowsOptionalFeature -Online -FeatureName $featureName -ErrorAction Stop
            $states[$featureName] = $feature.State
        } catch {
            $states[$featureName] = "Unavailable"
        }
    }

    $computer = Get-CimInstance Win32_ComputerSystem
    $cpu = Get-CimInstance Win32_Processor | Select-Object -First 1

    [pscustomobject]@{
        HypervisorPlatformState       = $states["HypervisorPlatform"]
        VirtualMachinePlatformState   = $states["VirtualMachinePlatform"]
        MicrosoftHyperVAllState       = $states["Microsoft-Hyper-V-All"]
        HypervisorPresent             = $computer.HypervisorPresent
        CpuName                       = $cpu.Name
        VirtualizationFirmwareEnabled = $cpu.VirtualizationFirmwareEnabled
        SLAT                          = $cpu.SecondLevelAddressTranslationExtensions
        VMMonitorModeExtensions       = $cpu.VMMonitorModeExtensions
    }
}

function Assert-RunPrereqs {
    $state = Get-HypervisorState
    $featureReady = ($state.HypervisorPlatformState -eq "Enabled") -or ($state.MicrosoftHyperVAllState -eq "Enabled")

    if ($featureReady -and $state.HypervisorPresent) {
        return
    }

    $message = @(
        "Windows hypervisor prerequisites are not ready for crosvm."
        "HypervisorPlatformState     : $($state.HypervisorPlatformState)"
        "VirtualMachinePlatformState : $($state.VirtualMachinePlatformState)"
        "MicrosoftHyperVAllState     : $($state.MicrosoftHyperVAllState)"
        "HypervisorPresent           : $($state.HypervisorPresent)"
        ""
        "Enable the required Windows features in an elevated PowerShell session, then reboot:"
        "Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V-All,HypervisorPlatform,VirtualMachinePlatform -All"
        "Restart-Computer"
    ) -join [Environment]::NewLine

    throw $message
}

function Set-CommonEnvironment {
    param(
        [string]$BinDir,
        [string]$LibDir,
        [string]$VirtmgrExe,
        [string]$CrosvmExe,
        [string]$ServiceDirectory,
        [string]$ApexRoot,
        [string]$SystemRoot,
        [string]$SystemExtRoot,
        [string]$TracePath,
        [string]$VmclientTracePath,
        [string]$DebugPolicyPath,
        [string]$TempDirectory
    )

    $env:PATH = "$BinDir;$LibDir;$env:PATH"
    $env:VIRTMGR_PATH = $VirtmgrExe
    $env:VIRTMGR_CROSVM_PATH = $CrosvmExe
    if ($ServiceDirectory) {
        New-Item -ItemType Directory -Force -Path $ServiceDirectory | Out-Null
        $env:VIRTMGR_SERVICE_DIR = $ServiceDirectory
    } else {
        Remove-Item Env:VIRTMGR_SERVICE_DIR -ErrorAction SilentlyContinue
    }
    $env:VIRTMGR_APEX_ROOT = $ApexRoot
    $env:VIRTMGR_SYSTEM_ROOT = $SystemRoot
    $env:VIRTMGR_SYSTEM_EXT_ROOT = $SystemExtRoot
    $env:ANDROID_PROP_RO_BUILD_VERSION_SDK = "35"

    if ($TracePath) {
        $env:VIRTMGR_TRACE_FILE = $TracePath
    } else {
        Remove-Item Env:VIRTMGR_TRACE_FILE -ErrorAction SilentlyContinue
    }

    if ($VmclientTracePath) {
        $env:VMCLIENT_TRACE_FILE = $VmclientTracePath
    } else {
        Remove-Item Env:VMCLIENT_TRACE_FILE -ErrorAction SilentlyContinue
    }

    if ($DebugPolicyPath) {
        $env:VIRTMGR_DEBUG_POLICY_JSON = $DebugPolicyPath
    } else {
        Remove-Item Env:VIRTMGR_DEBUG_POLICY_JSON -ErrorAction SilentlyContinue
    }

    if ($TempDirectory) {
        $env:TEMP = $TempDirectory
        $env:TMP = $TempDirectory
    }

    if ($KeepTemp) {
        $env:VIRTMGR_KEEP_TEMP = "1"
    } else {
        Remove-Item Env:VIRTMGR_KEEP_TEMP -ErrorAction SilentlyContinue
    }

    if ($CaptureGuestConsole) {
        $env:VIRTMGR_CAPTURE_GUEST_CONSOLE = "1"
    } else {
        Remove-Item Env:VIRTMGR_CAPTURE_GUEST_CONSOLE -ErrorAction SilentlyContinue
    }

    if ($CaptureCrosvmStdio) {
        $env:VIRTMGR_CAPTURE_CROSVM_STDIO = "1"
    } else {
        Remove-Item Env:VIRTMGR_CAPTURE_CROSVM_STDIO -ErrorAction SilentlyContinue
    }
}

function Add-CommonVmArgs {
    param([System.Collections.Generic.List[string]]$Args)

    if ($Name) {
        $Args.Add("--name")
        $Args.Add($Name)
    }

    if ($Protected) {
        throw "Protected VM is not supported on the current Windows x86_64 + WHPX path. Run without -Protected."
    }
}

function Get-VirtmgrServiceStatePath {
    param([string]$ResolvedServiceRoot)
    return Join-Path $ResolvedServiceRoot "virtmgr-service.state"
}

function Get-VirtmgrServiceTracePath {
    param([string]$ResolvedServiceRoot)
    return Join-Path $ResolvedServiceRoot "virtmgr-trace.log"
}

function Read-VirtmgrServiceState {
    param([string]$ResolvedServiceRoot)

    $statePath = Get-VirtmgrServiceStatePath -ResolvedServiceRoot $ResolvedServiceRoot
    if (-not (Test-Path -LiteralPath $statePath)) {
        return $null
    }

    $values = @{}
    foreach ($line in Get-Content -LiteralPath $statePath) {
        $parts = $line -split "=", 2
        if ($parts.Length -eq 2) {
            $values[$parts[0].Trim()] = $parts[1].Trim()
        }
    }

    if (-not $values.ContainsKey("pid") -or -not $values.ContainsKey("rpc_port")) {
        return $null
    }

    [pscustomobject]@{
        StatePath = $statePath
        Pid = [int]$values["pid"]
        RpcPort = [int]$values["rpc_port"]
    }
}

function Stop-VirtmgrService {
    param([string]$ResolvedServiceRoot)

    $state = Read-VirtmgrServiceState -ResolvedServiceRoot $ResolvedServiceRoot
    $statePath = Get-VirtmgrServiceStatePath -ResolvedServiceRoot $ResolvedServiceRoot
    if ($null -eq $state) {
        Remove-Item -LiteralPath $statePath -Force -ErrorAction SilentlyContinue
        Write-Host "No persistent virtmgr service is registered."
        return
    }

    $process = Get-Process -Id $state.Pid -ErrorAction SilentlyContinue
    if ($process) {
        Stop-Process -Id $state.Pid -Force
        Write-Host "Stopped virtmgr service PID $($state.Pid)."
    } else {
        Write-Host "Persistent virtmgr PID $($state.Pid) is not running."
    }
    Remove-Item -LiteralPath $statePath -Force -ErrorAction SilentlyContinue
}

function Show-VirtmgrServiceStatus {
    param([string]$ResolvedServiceRoot)

    $state = Read-VirtmgrServiceState -ResolvedServiceRoot $ResolvedServiceRoot
    $tracePath = Get-VirtmgrServiceTracePath -ResolvedServiceRoot $ResolvedServiceRoot
    $process = $null
    if ($state) {
        $process = Get-Process -Id $state.Pid -ErrorAction SilentlyContinue
    }

    [pscustomobject]@{
        ServiceRoot = $ResolvedServiceRoot
        StateFile = Get-VirtmgrServiceStatePath -ResolvedServiceRoot $ResolvedServiceRoot
        TraceFile = $tracePath
        Registered = ($null -ne $state)
        Pid = if ($state) { $state.Pid } else { $null }
        RpcPort = if ($state) { $state.RpcPort } else { $null }
        Running = ($null -ne $process)
    } | Format-List
}

if ([string]::IsNullOrWhiteSpace($DistRoot)) {
    if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
        $RepoRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
    }
    $DistRoot = Join-Path $RepoRoot "out\dist"
}

$BinDir = Join-Path $DistRoot "windows\bin"
$LibDir = Join-Path $DistRoot "windows\lib"
$VmExe = Join-Path $BinDir "vm.exe"
$VirtmgrExe = Join-Path $BinDir "virtmgr.exe"
$CrosvmExe = Join-Path $BinDir "crosvm.exe"
$ApexTreeRoot = Join-Path $DistRoot "apex_dir"

Require-Path -Path $RepoRoot -Label "Repository root"
Require-Path -Path $DistRoot -Label "dist root"
Require-Path -Path $BinDir -Label "Windows dist bin directory"
Require-Path -Path $LibDir -Label "Windows dist lib directory"
Require-Path -Path $VmExe -Label "vm.exe"
Require-Path -Path $VirtmgrExe -Label "virtmgr.exe"
Require-Path -Path $CrosvmExe -Label "crosvm.exe"
Require-Path -Path $ApexTreeRoot -Label "Windows apex tree root"

Ensure-WindowsApexLayout -ResolvedApexTreeRoot $ApexTreeRoot
$ApexRoot = Join-Path $ApexTreeRoot "apex"
$ComAndroidVirt = Join-Path $ApexRoot "com.android.virt"
$SystemRoot = Join-Path $ApexTreeRoot "system"
$SystemExtRoot = Join-Path $ApexTreeRoot "system_ext"
Require-Path -Path $ApexRoot -Label "mounted apex root"
Require-Path -Path (Join-Path $ApexRoot "com.android.virt") -Label "mounted com.android.virt apex"

if ([string]::IsNullOrWhiteSpace($ServiceRoot)) {
    $ServiceRoot = Join-Path $DistRoot "logs\windows-virtmgr-service"
}
$ServiceTraceFile = Get-VirtmgrServiceTracePath -ResolvedServiceRoot $ServiceRoot

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
if ([string]::IsNullOrWhiteSpace($LogDir)) {
    $LogDir = Join-Path $DistRoot ("logs\windows-{0}-{1}" -f $Command, $timestamp)
}

if ([string]::IsNullOrWhiteSpace($WorkDir)) {
    $WorkDir = Join-Path $LogDir "work"
}

if ([string]::IsNullOrWhiteSpace($TempRoot)) {
    $TempRoot = Join-Path $LogDir "temp"
}

New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null
New-Item -ItemType Directory -Force -Path $TempRoot | Out-Null

if ([string]::IsNullOrWhiteSpace($Console)) {
    $Console = Join-Path $LogDir "vm-console.txt"
}
if ([string]::IsNullOrWhiteSpace($ConsoleIn)) {
    $ConsoleIn = Join-Path $LogDir "vm-console-in.txt"
}
if ([string]::IsNullOrWhiteSpace($GuestLog)) {
    $GuestLog = Join-Path $LogDir "guest-log.txt"
}
if ([string]::IsNullOrWhiteSpace($TraceFile)) {
    if ($PersistVirtmgr) {
        $TraceFile = $ServiceTraceFile
    } else {
        $TraceFile = Join-Path $LogDir "virtmgr-trace.log"
    }
}
if ([string]::IsNullOrWhiteSpace($VmclientTraceFile)) {
    $VmclientTraceFile = Join-Path $LogDir "vmclient-trace.log"
}

$RunLogFile = Join-Path $LogDir ("vm-{0}.log" -f $Command)

if ($Command -in @("run-microdroid", "run-app", "run")) {
    New-ParentDirectory -Path $Console
    New-ParentDirectory -Path $ConsoleIn
    New-ParentDirectory -Path $GuestLog
    if (-not (Test-Path -LiteralPath $ConsoleIn)) {
        New-Item -ItemType File -Force -Path $ConsoleIn | Out-Null
    }
}

if ($DebugPolicyJson) {
    Require-Path -Path $DebugPolicyJson -Label "Debug policy JSON"
}

Set-CommonEnvironment -BinDir $BinDir -LibDir $LibDir -VirtmgrExe $VirtmgrExe -CrosvmExe $CrosvmExe -ServiceDirectory $(if ($PersistVirtmgr) { $ServiceRoot } else { "" }) -ApexRoot $ApexRoot -SystemRoot $SystemRoot -SystemExtRoot $SystemExtRoot -TracePath $TraceFile -VmclientTracePath $VmclientTraceFile -DebugPolicyPath $DebugPolicyJson -TempDirectory $TempRoot

$commandArgs = [System.Collections.Generic.List[string]]::new()

switch ($Command) {
    "validate-prereqs" {
        $state = Get-HypervisorState
        $state | Format-List | Tee-Object -FilePath $RunLogFile
        if ((($state.HypervisorPlatformState -eq "Enabled") -or ($state.MicrosoftHyperVAllState -eq "Enabled")) -and $state.HypervisorPresent) {
            exit 0
        }
        exit 1
    }
    "run-microdroid" {
        if (-not $SkipHypervisorCheck -and -not $DryRun) {
            Assert-RunPrereqs
        }
        Add-CommonVmArgs -Args $commandArgs
        $commandArgs.Add("run-microdroid")
        $commandArgs.Add("--work-dir")
        $commandArgs.Add($WorkDir)
        $commandArgs.Add("--console")
        $commandArgs.Add($Console)
        $commandArgs.Add("--console-in")
        $commandArgs.Add($ConsoleIn)
        $commandArgs.Add("--log")
        $commandArgs.Add($GuestLog)
    }
    "run-app" {
        if (-not $SkipHypervisorCheck -and -not $DryRun) {
            Assert-RunPrereqs
        }
        if ([string]::IsNullOrWhiteSpace($Apk)) {
            $Apk = Join-Path $ComAndroidVirt "app\EmptyPayloadApp@AP4A.250205.002\EmptyPayloadApp.apk"
        }
        if ([string]::IsNullOrWhiteSpace($Idsig)) {
            $Idsig = Join-Path $WorkDir "app.idsig"
        }
        if ([string]::IsNullOrWhiteSpace($Instance)) {
            $Instance = Join-Path $WorkDir "instance.img"
        }

        Require-Path -Path $Apk -Label "VM payload APK"
        New-ParentDirectory -Path $Idsig
        New-ParentDirectory -Path $Instance

        Add-CommonVmArgs -Args $commandArgs
        $commandArgs.Add("run-app")
        $commandArgs.Add($Apk)
        $commandArgs.Add($Idsig)
        $commandArgs.Add($Instance)
        $commandArgs.Add("--payload-binary-name")
        $commandArgs.Add($PayloadBinaryName)
        $commandArgs.Add("--console")
        $commandArgs.Add($Console)
        $commandArgs.Add("--console-in")
        $commandArgs.Add($ConsoleIn)
        $commandArgs.Add("--log")
        $commandArgs.Add($GuestLog)
    }
    "run" {
        if (-not $SkipHypervisorCheck -and -not $DryRun) {
            Assert-RunPrereqs
        }
        if ([string]::IsNullOrWhiteSpace($Config)) {
            $Config = Join-Path $RepoRoot "scripts\microdroid_windows_raw.json"
        }

        Require-Path -Path $Config -Label "VM config JSON"

        Add-CommonVmArgs -Args $commandArgs
        $commandArgs.Add("run")
        $commandArgs.Add($Config)
        $commandArgs.Add("--console")
        $commandArgs.Add($Console)
        $commandArgs.Add("--console-in")
        $commandArgs.Add($ConsoleIn)
        $commandArgs.Add("--log")
        $commandArgs.Add($GuestLog)
    }
    "info" {
        $commandArgs.Add("info")
    }
    "list" {
        $commandArgs.Add("list")
    }
    "console" {
        $commandArgs.Add("console")
        if ($Cid -gt 0) {
            $commandArgs.Add($Cid.ToString())
        }
    }
    "check-feature-enabled" {
        $commandArgs.Add("check-feature-enabled")
        $commandArgs.Add($Feature)
    }
    "create-partition" {
        if ([string]::IsNullOrWhiteSpace($PartitionPath)) {
            $PartitionPath = Join-Path $WorkDir "writable.img"
        }
        if ($PartitionSize -eq 0) {
            $PartitionSize = 1048576
        }

        New-ParentDirectory -Path $PartitionPath

        $commandArgs.Add("create-partition")
        $commandArgs.Add($PartitionPath)
        $commandArgs.Add($PartitionSize.ToString())
        if ($PartitionType -and ($PartitionType -ne "raw")) {
            $commandArgs.Add("--type")
            $commandArgs.Add($PartitionType)
        }
    }
    "create-idsig" {
        if ([string]::IsNullOrWhiteSpace($Apk)) {
            $Apk = Join-Path $ComAndroidVirt "app\EmptyPayloadApp@AP4A.250205.002\EmptyPayloadApp.apk"
        }
        if ([string]::IsNullOrWhiteSpace($OutputPath)) {
            $OutputPath = Join-Path $WorkDir "app.idsig"
        }

        Require-Path -Path $Apk -Label "VM payload APK"
        New-ParentDirectory -Path $OutputPath

        $commandArgs.Add("create-idsig")
        $commandArgs.Add($Apk)
        $commandArgs.Add($OutputPath)
    }
    "service-status" {
        Show-VirtmgrServiceStatus -ResolvedServiceRoot $ServiceRoot | Tee-Object -FilePath $RunLogFile
        exit 0
    }
    "stop-service" {
        Stop-VirtmgrService -ResolvedServiceRoot $ServiceRoot | Tee-Object -FilePath $RunLogFile
        exit 0
    }
}

if ($VmArgs) {
    foreach ($arg in $VmArgs) {
        $commandArgs.Add($arg)
    }
}

Write-Host "Command            : $Command"
Write-Host "RepoRoot           : $RepoRoot"
Write-Host "DistRoot           : $DistRoot"
Write-Host "WorkDir            : $WorkDir"
Write-Host "LogDir             : $LogDir"
Write-Host "ServiceRoot        : $ServiceRoot"
Write-Host "PersistVirtmgr     : $PersistVirtmgr"
Write-Host "TempRoot           : $TempRoot"
Write-Host "vm.exe             : $VmExe"
Write-Host "virtmgr.exe        : $VirtmgrExe"
Write-Host "crosvm.exe         : $CrosvmExe"
Write-Host "TraceFile          : $TraceFile"
if ($PersistVirtmgr) {
    Write-Host "ServiceTraceFile   : $ServiceTraceFile"
}
Write-Host "VmclientTraceFile  : $VmclientTraceFile"
if ($DebugPolicyJson) {
    Write-Host "DebugPolicyJson    : $DebugPolicyJson"
}
Write-Host "RunLog             : $RunLogFile"
if ($Command -in @("run-microdroid", "run-app", "run")) {
    Write-Host "GuestLog           : $GuestLog"
    Write-Host "Console            : $Console"
    Write-Host "ConsoleIn          : $ConsoleIn"
}
Write-Host "KeepTemp           : $KeepTemp"
Write-Host "CaptureGuestConsole: $CaptureGuestConsole"
Write-Host "CaptureCrosvmStdio : $CaptureCrosvmStdio"
if ($TimeoutSecs -gt 0) {
    Write-Host "TimeoutSecs        : $TimeoutSecs"
}
Write-Host "Resolved vm args   : `"$VmExe`" $($commandArgs -join ' ')"

if ($DryRun) {
    exit 0
}

$stdoutLog = Join-Path $LogDir "vm-run-stdout.tmp.log"
$stderrLog = Join-Path $LogDir "vm-run-stderr.tmp.log"
Remove-Item -LiteralPath $stdoutLog, $stderrLog -Force -ErrorAction SilentlyContinue

$vmProc = Start-Process -FilePath $VmExe `
    -ArgumentList $commandArgs `
    -NoNewWindow `
    -PassThru `
    -RedirectStandardOutput $stdoutLog `
    -RedirectStandardError $stderrLog

$timedOut = $false
if ($null -ne $vmProc) {
    if ($TimeoutSecs -gt 0) {
        $finished = Wait-Process -Id $vmProc.Id -Timeout $TimeoutSecs -ErrorAction SilentlyContinue
        if ($null -eq $finished) {
            $timedOut = $true
            Stop-Process -Id $vmProc.Id -Force -ErrorAction SilentlyContinue
            $null = Wait-Process -Id $vmProc.Id -ErrorAction SilentlyContinue
            Start-Sleep -Seconds 1
        }
    } else {
        $null = Wait-Process -Id $vmProc.Id
    }
    $vmProc.Refresh()
}
$exitCode = if ($timedOut) { 124 } elseif (($null -ne $vmProc) -and $vmProc.HasExited) { $vmProc.ExitCode } else { 0 }

$stdoutContent = Read-TextFileWithRetry -Path $stdoutLog
$stderrContent = Read-TextFileWithRetry -Path $stderrLog
$combined = $stdoutContent
if ($stdoutContent -and $stderrContent) {
    $combined += [Environment]::NewLine
}
$combined += $stderrContent
[System.IO.File]::WriteAllText($RunLogFile, $combined)
if ($combined) {
    Write-Host $combined -NoNewline
}
Remove-Item -LiteralPath $stdoutLog, $stderrLog -Force -ErrorAction SilentlyContinue

Write-Host ""
if ($timedOut) {
    Write-Host "vm.exe timed out after $TimeoutSecs seconds"
}
Write-Host "vm.exe exit code   : $exitCode"
Write-Host "Artifacts written to: $LogDir"

exit $exitCode
