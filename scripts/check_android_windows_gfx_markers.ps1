param(
    [Parameter(Mandatory = $true)][string]$LogDir
)

$ErrorActionPreference = "Stop"

$Logcat = Join-Path $LogDir "logcat-hvc2.txt"
$KernelLog = Join-Path $LogDir "hvc.txt"
$StderrLog = Join-Path $LogDir "stderr.txt"

if (-not (Test-Path $Logcat) -or -not (Test-Path $KernelLog) -or -not (Test-Path $StderrLog)) {
    throw "Missing Android/gfxstream logs under $LogDir"
}

function Require-Fixed {
    param([string]$Pattern, [string]$File, [string]$Label)
    if (-not (Select-String -Path $File -Pattern $Pattern -SimpleMatch -Quiet)) {
        throw "Missing marker: $Label"
    }
}

function Require-Regex {
    param([string]$Pattern, [string[]]$File, [string]$Label)
    $found = $false
    foreach ($path in $File) {
        if (Select-String -Path $path -Pattern $Pattern -Quiet) {
            $found = $true
            break
        }
    }
    if (-not $found) {
        throw "Missing marker: $Label"
    }
}

function Reject-Regex {
    param([string]$Pattern, [string]$File, [string]$Label)
    if (Select-String -Path $File -Pattern $Pattern -Quiet) {
        throw "Unexpected marker: $Label"
    }
}

& "$PSScriptRoot\check_android_windows_markers.ps1" -LogDir $LogDir

Require-Fixed -Pattern "SurfaceFlinger: Boot is finished" -File $Logcat -Label "SurfaceFlinger boot finished"
Require-Regex -Pattern "RenderEngine: renderer  : ANGLE .*Vulkan" -File $Logcat -Label "RenderEngine uses ANGLE over Vulkan"
Require-Fixed -Pattern "RenderEngine: version   : OpenGL ES" -File $Logcat -Label "RenderEngine OpenGL ES version"

Require-Fixed -Pattern "Gfxstream feature GuestVulkanOnly enabled" -File $StderrLog -Label "gfxstream GuestVulkanOnly enabled"
Require-Regex -Pattern "Gfxstream feature VulkanAllocateHostMemory (enabled|disabled)" -File $StderrLog `
    -Label "gfxstream VulkanAllocateHostMemory state logged"
Require-Fixed -Pattern "stream_renderer_init Gfxstream initialized successfully!" -File $StderrLog `
    -Label "gfxstream initialized"
Require-Fixed -Pattern "Graphics Adapter" -File $StderrLog -Label "host graphics adapter selected"
if (Select-String -Path $StderrLog -Pattern "VulkanAllocateHostMemory: enabled" -Quiet) {
    Require-Regex -Pattern "Coherent host memory probe result: typeBits=0x" -File $StderrLog `
        -Label "coherent host memory probe"
    Require-Fixed -Pattern "ExternalBlob: enabled" -File $StderrLog -Label "ExternalBlob enabled for coherent blobs"
}
Require-Regex -Pattern "(Created VkDevice:.*application:surfaceflinger engine:ANGLE|RenderEngine: renderer  : ANGLE .*Vulkan)" `
    -File @($StderrLog, $Logcat) -Label "surfaceflinger ANGLE Vulkan path"
Require-Regex -Pattern "(update_scanout_resource type=Scanout|create_surface:.*type=Scanout)" -File $StderrLog `
    -Label "scanout resource updates"
Require-Fixed -Pattern "flush_resource: id=" -File $StderrLog -Label "scanout resource flushes"

Reject-Regex -Pattern "eglMakeCurrent failed|null ctx|vkGetMemoryHostPointerPropertiesEXT|Invalid device|\[Vulkan Loader\] ERROR|Segmentation fault|SIGSEGV" `
    -File $StderrLog -Label "host gfxstream/ANGLE failure"
Reject-Regex -Pattern "RenderEngine:.*(failed|error)|SurfaceFlinger:.*(crash|fatal)|HWComposer.*(crash|fatal)|eglMakeCurrent failed|null ctx" `
    -File $Logcat -Label "guest graphics failure"

Write-Host "GFX marker check passed: android-windows gfxstream+ANGLE"
