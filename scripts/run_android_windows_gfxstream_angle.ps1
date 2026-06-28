param(
    [string]$ArtifactDir = "",
    [string]$WorkDir = "",
    [string]$LogDir = "",
    [int]$Mem = 8192,
    [int]$Cpus = 4,
    [int]$Cid = 100,
    [int]$TimeoutSecs = 0,
    [ValidateSet("run", "run-mp")]
    [string]$RunMode = "run-mp",
    [switch]$RefreshImages,
    [switch]$RebuildAggregate,
    [switch]$ResetBootState,
    [switch]$NoRun,
    [switch]$DryRun,
    [switch]$UseSwiftShader,
    [switch]$ConservativeWhpx,
    [switch]$GpuHostVisibleCoherent,
    [switch]$FullHvc,
    [switch]$NoNetwork,
    [switch]$NoBluetooth,
    [switch]$NoNfc,
    [switch]$NoModem,
    [string]$AospHostBin = "",
    [string]$HostBinDir = "",
    [int]$BtHciPort = 7300,
    [int]$CasimirNciPort = 7800,
    [int]$CasimirRfPort = 7900,
    [string]$RilGateway = "192.168.97.1",
    [string]$RilIpaddr = "192.168.97.2",
    [int]$RilPrefixlen = 30,
    [string]$RilDns = "8.8.8.8",
    [int]$ModemInstanceNum = 1,
    [int]$ModemSimType = 1,
    [int]$ModemBasePort = 9600
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$script:HostDaemonProcesses = @()
if (-not $WorkDir) { $WorkDir = Join-Path $RepoRoot "out\android-windows" }
if (-not $LogDir) { $LogDir = Join-Path $RepoRoot "out\dist\logs\android-windows" }

function Find-DefaultArtifactDir {
    $root = "D:\bscp-vm-artifacts"
    if (-not (Test-Path $root)) {
        throw "Artifact root not found: $root"
    }

    $candidates = Get-ChildItem -Path $root -Directory -Filter "bscp-vm-artifacts-*" |
        ForEach-Object {
            Join-Path $_.FullName "products\android\vsoc_x86_64\direct-linux"
        } |
        Where-Object {
            Test-Path (Join-Path $_ "aggregate_android.img")
        } |
        ForEach-Object {
            Get-Item $_
        } |
        Sort-Object LastWriteTime -Descending

    if (-not $candidates) {
        throw "No Android direct-linux image set found under $root"
    }
    return $candidates[0].FullName
}

function Copy-IfNeeded {
    param(
        [Parameter(Mandatory = $true)][string]$Source,
        [Parameter(Mandatory = $true)][string]$Destination
    )

    if (-not (Test-Path $Source)) {
        throw "Missing source file: $Source"
    }

    $src = Get-Item -LiteralPath $Source
    $copy = $RefreshImages -or -not (Test-Path $Destination)
    if (-not $copy) {
        $dst = Get-Item -LiteralPath $Destination
        $copy = $dst.Length -ne $src.Length
    }

    if ($copy) {
        Write-Host "copy $Source -> $Destination"
        Copy-Item -LiteralPath $Source -Destination $Destination -Force
    } else {
        Write-Host "reuse $Destination"
    }
}

function Join-WindowsArgumentList {
    param([string[]]$Arguments)

    $quoted = foreach ($arg in $Arguments) {
        if ($arg -ne "" -and $arg -notmatch '[\s"]') {
            $arg
            continue
        }

        $builder = New-Object System.Text.StringBuilder
        [void]$builder.Append('"')
        $backslashes = 0
        foreach ($ch in $arg.ToCharArray()) {
            if ($ch -eq '\') {
                $backslashes++
                continue
            }
            if ($ch -eq '"') {
                [void]$builder.Append('\' * (($backslashes * 2) + 1))
                [void]$builder.Append('"')
                $backslashes = 0
                continue
            }
            if ($backslashes -gt 0) {
                [void]$builder.Append('\' * $backslashes)
                $backslashes = 0
            }
            [void]$builder.Append($ch)
        }
        if ($backslashes -gt 0) {
            [void]$builder.Append('\' * ($backslashes * 2))
        }
        [void]$builder.Append('"')
        $builder.ToString()
    }

    return ($quoted -join " ")
}

function Resolve-HostBinary {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [string[]]$SearchDirs
    )

    foreach ($dir in $SearchDirs) {
        if (-not $dir) { continue }
        $candidate = Join-Path $dir $Name
        if ($Name -notmatch '\.') {
            $candidate = Join-Path $dir "$Name.exe"
        }
        if (Test-Path $candidate) {
            return (Resolve-Path $candidate).Path
        }
    }
    return $null
}

function Get-CfPipePath {
    param([string]$Name)
    return "\\.\pipe\bscp_cf_${Name}_$([System.Diagnostics.Process]::GetCurrentProcess().Id)"
}

function Get-CfModemVsockPort {
    param(
        [int]$GuestCid,
        [int]$BasePort = 9600
    )
    return $BasePort + $GuestCid - 3
}

function Get-CfBinderRpcVsockPipePath {
    param(
        [int]$GuestCid,
        [int]$Port
    )
    return "\\.\pipe\binder_rpc_vsock_${GuestCid}_${Port}"
}

function Start-CfHostDaemons {
    param(
        [string]$WorkDir,
        [string]$LogDir,
        [string[]]$SearchDirs,
        [bool]$EnableBluetooth,
        [bool]$EnableNfc,
        [bool]$EnableModem,
        [int]$GuestCid,
        [string]$BtOutPipe,
        [string]$BtInPipe,
        [string]$NfcOutPipe,
        [string]$NfcInPipe,
        [string]$RilGateway,
        [string]$RilIpaddr,
        [int]$RilPrefixlen,
        [string]$RilDns,
        [int]$ModemBasePort
    )

    $bridgeScript = Join-Path $PSScriptRoot "cf_hvc_tcp_bridge.ps1"
    if (-not (Test-Path $bridgeScript)) {
        throw "Missing bridge script: $bridgeScript"
    }

    if ($EnableBluetooth) {
        $rootCanal = Resolve-HostBinary -Name "root-canal" -SearchDirs $SearchDirs
        if (-not $rootCanal) {
            Write-Host "Warning: Bluetooth requested but root-canal not found; skipping BT daemons."
            $script:BluetoothEnabled = $false
        } else {
            $rootStdout = Join-Path $LogDir "root-canal.stdout.txt"
            $rootStderr = Join-Path $LogDir "root-canal.stderr.txt"
            $btStdout = Join-Path $LogDir "bt-bridge.stdout.txt"
            $btStderr = Join-Path $LogDir "bt-bridge.stderr.txt"
            $script:HostDaemonProcesses += Start-Process -FilePath $rootCanal `
                -ArgumentList @(
                    "--test_port=$($BtHciPort + 1)",
                    "--hci_port=$BtHciPort",
                    "--link_port=$($BtHciPort + 2)",
                    "--link_ble_port=$($BtHciPort + 3)"
                ) `
                -RedirectStandardOutput $rootStdout -RedirectStandardError $rootStderr -PassThru
            Start-Sleep -Seconds 1
            $script:HostDaemonProcesses += Start-Process -FilePath "powershell.exe" `
                -ArgumentList @(
                    "-NoProfile", "-ExecutionPolicy", "Bypass",
                    "-File", $bridgeScript,
                    "-PipeOut", $BtOutPipe,
                    "-PipeIn", $BtInPipe,
                    "-TcpPort", "$BtHciPort",
                    "-BufferSize", "2050"
                ) `
                -RedirectStandardOutput $btStdout -RedirectStandardError $btStderr -PassThru
            $script:BluetoothEnabled = $true
        }
    }

    if ($EnableNfc) {
        $casimir = Resolve-HostBinary -Name "casimir" -SearchDirs $SearchDirs
        if (-not $casimir) {
            Write-Host "Warning: NFC requested but casimir not found; skipping NFC daemons."
            $script:NfcEnabled = $false
        } else {
            $casimirStdout = Join-Path $LogDir "casimir.stdout.txt"
            $casimirStderr = Join-Path $LogDir "casimir.stderr.txt"
            $nfcStdout = Join-Path $LogDir "nfc-bridge.stdout.txt"
            $nfcStderr = Join-Path $LogDir "nfc-bridge.stderr.txt"
            $script:HostDaemonProcesses += Start-Process -FilePath $casimir `
                -ArgumentList @("--nci-port", "$CasimirNciPort", "--rf-port", "$CasimirRfPort") `
                -RedirectStandardOutput $casimirStdout -RedirectStandardError $casimirStderr -PassThru
            $script:HostDaemonProcesses += Start-Process -FilePath "powershell.exe" `
                -ArgumentList @(
                    "-NoProfile", "-ExecutionPolicy", "Bypass",
                    "-File", $bridgeScript,
                    "-PipeOut", $NfcOutPipe,
                    "-PipeIn", $NfcInPipe,
                    "-TcpPort", "$CasimirNciPort",
                    "-BufferSize", "1024"
                ) `
                -RedirectStandardOutput $nfcStdout -RedirectStandardError $nfcStderr -PassThru
            $script:NfcEnabled = $true
        }
    }

    if ($EnableModem) {
        $modemScript = Join-Path $PSScriptRoot "modem_simulator_windows.py"
        if (-not (Test-Path $modemScript)) {
            throw "Missing Windows modem simulator: $modemScript"
        }
        $modemStdout = Join-Path $LogDir "modem-simulator.stdout.txt"
        $modemStderr = Join-Path $LogDir "modem-simulator.stderr.txt"
        $modemPort = Get-CfModemVsockPort -GuestCid $GuestCid -BasePort $ModemBasePort
        $modemPipe = Get-CfBinderRpcVsockPipePath -GuestCid $GuestCid -Port $modemPort
        $modemArgs = @(
            $modemScript,
            "--guest-cid", "$GuestCid",
            "--base-port", "$ModemBasePort",
            "--ril-gateway", $RilGateway,
            "--ril-ipaddr", $RilIpaddr,
            "--ril-prefixlen", "$RilPrefixlen",
            "--ril-dns", $RilDns
        )
        Write-Host "Modem pipe:  $modemPipe (Python host modem, vsock port $modemPort)"
            $script:HostDaemonProcesses += Start-Process -FilePath "python" `
                -ArgumentList $modemArgs `
                -RedirectStandardOutput $modemStdout -RedirectStandardError $modemStderr -PassThru
            Start-Sleep -Seconds 1
            $script:ModemEnabled = $true
    }
}

function Stop-CfHostDaemons {
    foreach ($proc in $script:HostDaemonProcesses) {
        if ($proc -and -not $proc.HasExited) {
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        }
    }
    $script:HostDaemonProcesses = @()
}

if (-not $ArtifactDir) {
    $ArtifactDir = Find-DefaultArtifactDir
}
$ArtifactDir = (Resolve-Path $ArtifactDir).Path

if ($ResetBootState) {
    $RefreshImages = $true
    $RebuildAggregate = $true
    Write-Host "ResetBootState: refreshing artifact inputs and rebuilding aggregate from images/"
}

New-Item -ItemType Directory -Force -Path $WorkDir, $LogDir | Out-Null
$HvcDir = Join-Path $WorkDir "hvc"
New-Item -ItemType Directory -Force -Path $HvcDir | Out-Null

$files = @(
    "aggregate_android.img",
    "android_fstab.dt",
    "initrd_android.img",
    "kernel",
    "misc.img",
    "metadata.img",
    "factory_reset_protected.img",
    "android.dtb",
    "android_fstab_extra.cpio.lz4"
)

foreach ($name in $files) {
    if ($RebuildAggregate -and $name -eq "aggregate_android.img") {
        Write-Host "skip copy $name (RebuildAggregate will create it)"
        continue
    }
    if ($DryRun) {
        $src = Join-Path $ArtifactDir $name
        if (-not (Test-Path $src)) { throw "Missing source file: $src" }
        Write-Host "dry-run: would copy $src"
    } else {
        Copy-IfNeeded -Source (Join-Path $ArtifactDir $name) -Destination (Join-Path $WorkDir $name)
    }
}

if ($RebuildAggregate -and -not $DryRun -and -not $NoRun) {
    $productDir = Join-Path (Split-Path $ArtifactDir -Parent) "images"
    if (-not (Test-Path $productDir)) {
        throw "RebuildAggregate requires sibling images/ under $(Split-Path $ArtifactDir -Parent)"
    }
    $diskScript = Join-Path $PSScriptRoot "create_cf_android_disk.py"
    if (-not (Test-Path $diskScript)) {
        throw "Missing disk builder: $diskScript"
    }
    $aggregateOut = Join-Path $WorkDir "aggregate_android.img"
    $aggregateTmp = Join-Path $WorkDir "aggregate_android_new.img"
    Write-Host "Rebuilding aggregate disk from $productDir -> $aggregateOut"
    & python $diskScript `
        --product-dir $productDir `
        --misc-image (Join-Path $ArtifactDir "misc.img") `
        --metadata-image (Join-Path $ArtifactDir "metadata.img") `
        --frp-image (Join-Path $ArtifactDir "factory_reset_protected.img") `
        --output $aggregateTmp
    if ($LASTEXITCODE -ne 0) {
        throw "create_cf_android_disk.py failed with exit $LASTEXITCODE"
    }
    Move-Item -LiteralPath $aggregateTmp -Destination $aggregateOut -Force
}

$artifactHvc = Join-Path $ArtifactDir "hvc"
foreach ($port in @("keymaster", "gatekeeper", "bt", "gnss", "location", "confui", "uwb", "oemlock", "keymint", "nfc", "sensors", "mcu_control", "mcu_uart")) {
    $dst = Join-Path $HvcDir "$port.in"
    $src = Join-Path $artifactHvc "$port.in"
    if (Test-Path $src) {
        if ($DryRun) {
            Write-Host "dry-run: would copy $src"
        } else {
            Copy-IfNeeded -Source $src -Destination $dst
        }
    } elseif (-not (Test-Path $dst)) {
        if (-not $DryRun) {
            New-Item -ItemType File -Path $dst | Out-Null
        }
    }
}

$DistRoot = Join-Path $RepoRoot "out\dist\windows"
$BinDir = if ($HostBinDir) { (Resolve-Path $HostBinDir).Path } else { Join-Path $DistRoot "bin" }
$Crosvm = Join-Path $BinDir "crosvm.exe"
if (-not (Test-Path $Crosvm)) {
    throw "crosvm.exe not found: $Crosvm"
}

$angleCandidates = @(
    (Join-Path $DistRoot "gfx\angle"),
    $BinDir,
    (Join-Path $RepoRoot "..\angle\out\Release-GfxAngle-Clang")
)
$AngleDir = $angleCandidates | Where-Object {
    (Test-Path (Join-Path $_ "libEGL.dll")) -and (Test-Path (Join-Path $_ "libGLESv2.dll"))
} | Select-Object -First 1
if ($AngleDir) {
    $env:GFXSTREAM_ANGLE_ROOT = $AngleDir
    $env:PATH = "$AngleDir;$BinDir;$env:PATH"
} else {
    $env:PATH = "$BinDir;$env:PATH"
}
$env:GFXSTREAM_PATH = $BinDir

$GfxstreamBuildDll = Join-Path $RepoRoot "out\gfxstream_build_windows\libgfxstream_backend.dll"
if (Test-Path $GfxstreamBuildDll) {
    $buildTime = (Get-Item $GfxstreamBuildDll).LastWriteTimeUtc
    $distLib = Join-Path $BinDir "libgfxstream_backend.dll"
    $needsSync = -not (Test-Path $distLib) -or ((Get-Item $distLib).LastWriteTimeUtc -lt $buildTime)
    if ($needsSync) {
        Copy-Item -Force $GfxstreamBuildDll $distLib
        Copy-Item -Force $GfxstreamBuildDll (Join-Path $BinDir "gfxstream_backend.dll")
        Write-Host "Synced gfxstream backend DLLs from $GfxstreamBuildDll"
    }
}

if ($UseSwiftShader) {
    $swiftshaderIcd = Join-Path $BinDir "vk_swiftshader_icd.json"
    if (-not (Test-Path $swiftshaderIcd)) {
        throw "SwiftShader ICD not found: $swiftshaderIcd"
    }
    $env:VK_ICD_FILENAMES = $swiftshaderIcd
}

if ($ConservativeWhpx) {
    $Cpus = 1
    $env:CROSVM_WHPX_BLOCK_QUEUES = "1"
}

$script:BluetoothEnabled = -not $NoBluetooth
$script:NfcEnabled = -not $NoNfc
$script:ModemEnabled = -not $NoModem
$ModemVsockPort = Get-CfModemVsockPort -GuestCid $Cid -BasePort $ModemBasePort
$EnableNetwork = -not $NoNetwork
$BtOutPipe = Get-CfPipePath -Name "bt_out"
$BtInPipe = Get-CfPipePath -Name "bt_in"
$NfcOutPipe = Get-CfPipePath -Name "nfc_out"
$NfcInPipe = Get-CfPipePath -Name "nfc_in"
$HostBinSearchDirs = @(
    (Join-Path $RepoRoot "out\dist\windows\bin"),
    $AospHostBin
) | Where-Object { $_ -and (Test-Path $_) }

if ($script:BluetoothEnabled -and -not (Resolve-HostBinary -Name "root-canal" -SearchDirs $HostBinSearchDirs)) {
    Write-Host "Warning: root-canal not found; disabling Bluetooth HVC pipes."
    $script:BluetoothEnabled = $false
}
if ($script:NfcEnabled -and -not (Resolve-HostBinary -Name "casimir" -SearchDirs $HostBinSearchDirs)) {
    Write-Host "Warning: casimir not found; disabling NFC HVC pipes."
    $script:NfcEnabled = $false
}

$Disk = Join-Path $WorkDir "aggregate_android.img"
$Fstab = Join-Path $WorkDir "android_fstab.dt"
$Initrd = Join-Path $WorkDir "initrd_android.img"
if ($script:ModemEnabled -and -not $DryRun) {
    $bootconfigPatch = Join-Path $PSScriptRoot "patch_initrd_bootconfig.py"
    if (-not (Test-Path $bootconfigPatch)) {
        throw "Missing bootconfig patch script: $bootconfigPatch"
    }
    & python $bootconfigPatch --initrd $Initrd --set "androidboot.modem_simulator_ports=$ModemVsockPort"
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to patch initrd bootconfig for modem_simulator_ports=$ModemVsockPort"
    }
    Write-Host "Bootconfig:  androidboot.modem_simulator_ports=$ModemVsockPort"
}
$Kernel = Join-Path $WorkDir "kernel"
$KernelParams = "console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init"
if ($ConservativeWhpx) {
    $KernelParams = "$KernelParams clearcpuid=297"
}

if (-not $GpuHostVisibleCoherent -and $env:CROSVM_ANDROID_HOST_VISIBLE_COHERENT -eq "1") {
    $GpuHostVisibleCoherent = $true
}

$GpuExternalBlob = "false"
$GpuRendererFeatures = ""
if ($GpuHostVisibleCoherent) {
    # Match Linux --gpu-host-visible-coherent: exportable blobs for guest ResourceMapBlob
    # and VulkanAllocateHostMemory for coherent type enforcement via export allocation.
    $GpuExternalBlob = "true"
    $GpuRendererFeatures = ",renderer-features=VulkanAllocateHostMemory:enabled"
}

$GpuArg = "backend=gfxstream,displays=[[mode=windowed[1280,720],dpi=[320,320],refresh-rate=60]],context-types=gfxstream-vulkan:gfxstream-composer,angle=true,gles=false,vulkan=true,wsi=vk,external-blob=$GpuExternalBlob,udmabuf=false$GpuRendererFeatures"

$netArgs = @()
if ($EnableNetwork) {
    $prevEap = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $helpText = & $Crosvm run-mp --help 2>&1 | Out-String
    } finally {
        $ErrorActionPreference = $prevEap
    }
    if ($helpText -notmatch "--net") {
        Write-Host "Warning: crosvm was built without --net support; disabling dual virtio-net."
        $EnableNetwork = $false
    } else {
        $netArgs = @(
            "--net", "mac=00:1a:11:e0:cf:00,pci-address=00:01.1",
            "--net", "mac=00:1a:11:e1:cf:00,pci-address=00:01.2"
        )
    }
}

if (-not $NoRun -and -not $DryRun) {
    Get-ChildItem -LiteralPath $LogDir -File -Filter "*.txt" -ErrorAction SilentlyContinue |
        Remove-Item -Force -ErrorAction SilentlyContinue
}

$serialArgs = @(
    "--serial", "type=file,path=$(Join-Path $LogDir "serial.txt"),hardware=serial,num=1,earlycon=true",
    "--serial", "type=sink,hardware=serial,num=2",
    "--serial", "hardware=legacy-virtio-console,num=1,type=file,path=$(Join-Path $LogDir "hvc.txt"),console=true,pci-address=00:02.0"
)

if ($RunMode -ne "run-mp" -or $FullHvc) {
    $serialArgs += @(
        "--serial", "hardware=legacy-virtio-console,num=2,type=sink,pci-address=00:04.0",
        "--serial", "hardware=legacy-virtio-console,num=3,type=file,path=$(Join-Path $LogDir "logcat-hvc2.txt"),pci-address=00:05.0",
        "--serial", "hardware=legacy-virtio-console,num=4,type=file,path=$(Join-Path $LogDir "keymaster-hvc3.txt"),input=$(Join-Path $HvcDir "keymaster.in"),pci-address=00:06.0",
        "--serial", "hardware=legacy-virtio-console,num=5,type=file,path=$(Join-Path $LogDir "gatekeeper-hvc4.txt"),input=$(Join-Path $HvcDir "gatekeeper.in"),pci-address=00:07.0"
    )
    if ($script:BluetoothEnabled) {
        $serialArgs += @(
            "--serial", "hardware=legacy-virtio-console,num=6,type=file,path=$BtOutPipe,input=$BtInPipe,pci-address=00:08.0"
        )
    } else {
        $serialArgs += @(
            "--serial", "hardware=legacy-virtio-console,num=6,type=sink,pci-address=00:08.0"
        )
    }
    $serialArgs += @(
        "--serial", "hardware=legacy-virtio-console,num=7,type=sink,pci-address=00:09.0",
        "--serial", "hardware=legacy-virtio-console,num=8,type=sink,pci-address=00:0a.0",
        "--serial", "hardware=legacy-virtio-console,num=9,type=file,path=$(Join-Path $LogDir "confui-hvc8.txt"),input=$(Join-Path $HvcDir "confui.in"),pci-address=00:0b.0",
        "--serial", "hardware=legacy-virtio-console,num=10,type=file,path=$(Join-Path $LogDir "uwb-hvc9.txt"),input=$(Join-Path $HvcDir "uwb.in"),pci-address=00:0c.0",
        "--serial", "hardware=legacy-virtio-console,num=11,type=file,path=$(Join-Path $LogDir "oemlock-hvc10.txt"),input=$(Join-Path $HvcDir "oemlock.in"),pci-address=00:0d.0",
        "--serial", "hardware=legacy-virtio-console,num=12,type=file,path=$(Join-Path $LogDir "keymint-hvc11.txt"),input=$(Join-Path $HvcDir "keymint.in"),pci-address=00:0e.0"
    )
    if ($script:NfcEnabled) {
        $serialArgs += @(
            "--serial", "hardware=legacy-virtio-console,num=13,type=file,path=$NfcOutPipe,input=$NfcInPipe,pci-address=00:0f.0"
        )
    } else {
        $serialArgs += @(
            "--serial", "hardware=legacy-virtio-console,num=13,type=sink,pci-address=00:0f.0"
        )
    }
    $serialArgs += @(
        "--serial", "hardware=legacy-virtio-console,num=14,type=file,path=$(Join-Path $LogDir "sensors-hvc13.txt"),input=$(Join-Path $HvcDir "sensors.in"),pci-address=00:10.0",
        "--serial", "hardware=legacy-virtio-console,num=15,type=file,path=$(Join-Path $LogDir "mcu-control-hvc14.txt"),input=$(Join-Path $HvcDir "mcu_control.in"),pci-address=00:11.0",
        "--serial", "hardware=legacy-virtio-console,num=16,type=file,path=$(Join-Path $LogDir "mcu-uart-hvc15.txt"),input=$(Join-Path $HvcDir "mcu_uart.in"),pci-address=00:12.0"
    )
}

# Match scripts/run_android_linux.sh device order: GPU, NET, BLOCK, then serial/HVC.
$crosvmArgs = @(
    "--log-level", "info",
    $RunMode,
    "--disable-sandbox",
    "--cid", "$Cid",
    "--mem", "$Mem",
    "--cpus", "$Cpus",
    "--no-balloon",
    "--no-usb",
    "--gpu", $GpuArg
) + $netArgs + @(
    "--block", "path=$Disk,ro=false,lock=false,sparse=false,pci-address=00:03.0"
) + $serialArgs + @(
    "--android-fstab", $Fstab,
    "--initrd", $Initrd,
    "--params", $KernelParams,
    $Kernel
)

$commandFile = Join-Path $LogDir "crosvm-command.txt"
@(
    "ArtifactDir=$ArtifactDir",
    "WorkDir=$WorkDir",
    "LogDir=$LogDir",
    "GFXSTREAM_ANGLE_ROOT=$env:GFXSTREAM_ANGLE_ROOT",
    "VK_ICD_FILENAMES=$env:VK_ICD_FILENAMES",
    "CROSVM_WHPX_BLOCK_QUEUES=$env:CROSVM_WHPX_BLOCK_QUEUES",
    "GPU_HOST_VISIBLE_COHERENT=$GpuHostVisibleCoherent",
    "NETWORK_ENABLED=$EnableNetwork",
    "BLUETOOTH_ENABLED=$($script:BluetoothEnabled)",
    "NFC_ENABLED=$($script:NfcEnabled)",
    "MODEM_ENABLED=$($script:ModemEnabled)",
    "MODEM_VSOCK_PORT=$ModemVsockPort",
    "MODEM_PIPE=$(if ($script:ModemEnabled) { Get-CfBinderRpcVsockPipePath -GuestCid $Cid -Port $ModemVsockPort } else { '' })",
    "BT_OUT_PIPE=$BtOutPipe",
    "BT_IN_PIPE=$BtInPipe",
    "NFC_OUT_PIPE=$NfcOutPipe",
    "NFC_IN_PIPE=$NfcInPipe",
    "Initrd=$Initrd",
    "",
    $Crosvm,
    ($crosvmArgs | ForEach-Object { "  $_" })
) | Set-Content -Path $commandFile -Encoding ASCII

Write-Host "ArtifactDir: $ArtifactDir"
Write-Host "WorkDir:     $WorkDir"
Write-Host "LogDir:      $LogDir"
Write-Host "Command:     $commandFile"
Write-Host "ANGLE:       $env:GFXSTREAM_ANGLE_ROOT"
if ($env:VK_ICD_FILENAMES) { Write-Host "Vulkan ICD:  $env:VK_ICD_FILENAMES" }
Write-Host "CPU/Mem:     $Cpus vCPU, ${Mem}MiB"
Write-Host "Run mode:    $RunMode"
Write-Host "GPU coherent: $(if ($GpuHostVisibleCoherent) { 'enabled (VulkanAllocateHostMemory + external-blob=true)' } else { 'disabled' })"
Write-Host "Network:     $EnableNetwork"
Write-Host "Bluetooth:   $($script:BluetoothEnabled)"
Write-Host "NFC:         $($script:NfcEnabled)"
if ($script:ModemEnabled) {
    Write-Host "Modem:       enabled (Python host modem on binder_rpc pipe, port $ModemVsockPort)"
} else {
    Write-Host "Modem:       disabled (use -NoModem to suppress)"
}
Write-Host "Initrd:      $Initrd"
Write-Host "GPU:         $GpuArg"

if ($DryRun -or $NoRun) {
    exit 0
}

$stdout = Join-Path $LogDir "stdout.txt"
$stderr = Join-Path $LogDir "stderr.txt"
Remove-Item -LiteralPath $stdout, $stderr -Force -ErrorAction SilentlyContinue

if (-not $DryRun) {
    Start-CfHostDaemons -WorkDir $WorkDir -LogDir $LogDir -SearchDirs $HostBinSearchDirs `
        -EnableBluetooth $script:BluetoothEnabled -EnableNfc $script:NfcEnabled `
        -EnableModem $script:ModemEnabled -GuestCid $Cid `
        -BtOutPipe $BtOutPipe -BtInPipe $BtInPipe -NfcOutPipe $NfcOutPipe -NfcInPipe $NfcInPipe `
        -RilGateway $RilGateway -RilIpaddr $RilIpaddr -RilPrefixlen $RilPrefixlen -RilDns $RilDns `
        -ModemBasePort $ModemBasePort
}

Write-Host "Launching Android on crosvm; logs are under $LogDir"
try {
if ($TimeoutSecs -gt 0) {
    $argumentLine = Join-WindowsArgumentList -Arguments $crosvmArgs
    $process = Start-Process -FilePath $Crosvm -ArgumentList $argumentLine -PassThru `
        -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    $deadline = (Get-Date).AddSeconds($TimeoutSecs)
    while (-not $process.HasExited -and (Get-Date) -lt $deadline) {
        Start-Sleep -Seconds 1
    }
    if (-not $process.HasExited) {
        Write-Host "Timeout reached (${TimeoutSecs}s); stopping crosvm pid $($process.Id)"
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        exit 124
    }
    exit $process.ExitCode
}

& $Crosvm @crosvmArgs > $stdout 2> $stderr
exit $LASTEXITCODE
} finally {
    Stop-CfHostDaemons
}
