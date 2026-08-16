[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$RunDir,
    [Parameter(Mandatory)][string]$ResultPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Event {
    param([Parameter(Mandatory)][hashtable]$Fields)
    [Console]::WriteLine(($Fields | ConvertTo-Json -Compress))
}

function Fail-Evidence {
    param([Parameter(Mandatory)][string]$Message)
    Write-Event @{ event = "android17.gfxstream_evidence.failed"; state = "failed"; error = $Message }
    throw $Message
}

function Get-Sha256 {
    param([Parameter(Mandatory)][string]$Path)

    $stream = [IO.File]::Open(
        $Path,
        [IO.FileMode]::Open,
        [IO.FileAccess]::Read,
        [IO.FileShare]::ReadWrite
    )
    $sha256 = [Security.Cryptography.SHA256]::Create()
    try {
        return -join ($sha256.ComputeHash($stream) | ForEach-Object { $_.ToString("x2") })
    }
    finally {
        $sha256.Dispose()
        $stream.Dispose()
    }
}

$resolvedRun = (Resolve-Path -LiteralPath $RunDir).Path
$stderrPath = Join-Path $resolvedRun "stderr.txt"
$logcatPath = Join-Path $resolvedRun "logcat-hvc2.txt"
$uartPath = Join-Path $resolvedRun "uart.txt"
$argsPath = Join-Path $resolvedRun "crosvm-args.txt"
$sensorPath = Join-Path $resolvedRun "sensors-host.stdout.txt"

Write-Event @{ event = "android17.gfxstream_evidence.started"; state = "started"; run_dir = $resolvedRun }
foreach ($path in @($stderrPath, $logcatPath, $uartPath, $argsPath, $sensorPath)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        Fail-Evidence "required evidence file is missing: $path"
    }
}

$stderr = Get-Content -LiteralPath $stderrPath -Raw
$logcat = Get-Content -LiteralPath $logcatPath -Raw
$uart = Get-Content -LiteralPath $uartPath -Raw
$arguments = Get-Content -LiteralPath $argsPath -Raw
$sensors = Get-Content -LiteralPath $sensorPath -Raw

foreach ($required in @(
    "AngleIndirect: enabled",
    "ExternalBlob: enabled",
    "GuestVulkanOnly: enabled",
    "Vulkan: enabled",
    "VulkanAllocateHostMemory: enabled",
    "VulkanNativeSwapchain: enabled",
    "Added library: vulkan-1.dll",
    "Selecting Vulkan device:"
)) {
    if (-not $stderr.Contains($required)) {
        Fail-Evidence "gfxstream evidence is missing: $required"
    }
}

foreach ($forbidden in @(
    "failed to build rutabaga",
    "rutabaga component failed",
    "error -22",
    "VK_ERROR_INITIALIZATION_FAILED",
    "Display initialize error",
    "panicked at"
)) {
    if ($stderr.Contains($forbidden)) {
        Fail-Evidence "gfxstream failure marker is present: $forbidden"
    }
}

foreach ($required in @(
    "SurfaceFlinger: Boot is finished",
    "processing action (sys.boot_completed=1)",
    "Service 'bootanim'",
    "exited with status 0",
    "CrosvmDisplay"
)) {
    if (-not $logcat.Contains($required)) {
        Fail-Evidence "Android 17 boot evidence is missing: $required"
    }
}

$fingerprintPattern = 'generic/aosp_cf_x86_64_only_phone/vsoc_x86_64_only:(?<release>[0-9]+)/(?<build_id>[^/]+)/(?<incremental>[0-9]+):userdebug/test-keys'
$fingerprintMatch = [regex]::Match($uart, $fingerprintPattern)
if (-not $fingerprintMatch.Success) {
    Fail-Evidence "Android Guest fingerprint is missing from UART evidence"
}
$androidRelease = $fingerprintMatch.Groups["release"].Value
$buildId = $fingerprintMatch.Groups["build_id"].Value
$buildIncremental = $fingerprintMatch.Groups["incremental"].Value
if ($androidRelease -ne "17") {
    Fail-Evidence "expected Android release 17, found $androidRelease"
}

foreach ($required in @(
    "backend=gfxstream",
    "context-types=gfxstream-vulkan:gfxstream-composer",
    "angle=true",
    "vulkan=true",
    "external-blob=true",
    "VulkanAllocateHostMemory:enabled"
)) {
    if (-not $arguments.Contains($required)) {
        Fail-Evidence "crosvm GPU argument is missing: $required"
    }
}
if (-not $sensors.Contains("sensors_simulator_host: HAL activated")) {
    Fail-Evidence "Android 17 split-sensors host did not activate"
}

$resultFullPath = [IO.Path]::GetFullPath($ResultPath)
$resultDirectory = Split-Path -Parent $resultFullPath
[void](New-Item -ItemType Directory -Force -Path $resultDirectory)
$stderrArtifact = Join-Path $resultDirectory "android17-gfxstream-stderr.txt"
Copy-Item -LiteralPath $stderrPath -Destination $stderrArtifact -Force

$result = [ordered]@{
    schema_version = 1
    generated_at = (Get-Date).ToUniversalTime().ToString("o")
    source_run_dir = $resolvedRun
    guest_fingerprint = $fingerprintMatch.Value
    android_release = $androidRelease
    build_id = $buildId
    build_incremental = $buildIncremental
    sys_boot_completed = "1"
    boot_animation = "stopped"
    surfaceflinger_display = "CrosvmDisplay"
    gfxstream_backend = "gfxstream"
    gfxstream_context_types = @("gfxstream-vulkan", "gfxstream-composer")
    effective_features = @(
        "AngleIndirect",
        "ExternalBlob",
        "GuestVulkanOnly",
        "Vulkan",
        "VulkanAllocateHostMemory",
        "VulkanNativeSwapchain"
    )
    vulkan_loader = "bundled vulkan-1.dll"
    stderr_sha256 = Get-Sha256 -Path $stderrPath
    logcat_sha256 = Get-Sha256 -Path $logcatPath
    uart_sha256 = Get-Sha256 -Path $uartPath
}
[IO.File]::WriteAllText(
    $resultFullPath,
    (($result | ConvertTo-Json -Depth 6) + "`n"),
    [Text.UTF8Encoding]::new($false)
)
Write-Event @{
    event = "android17.gfxstream_evidence.succeeded"
    state = "succeeded"
    result = $resultFullPath
    sys_boot_completed = "1"
    display = "CrosvmDisplay"
}
