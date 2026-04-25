param(
    [string]$RepoRoot = "",
    [string]$WorkDir = "",
    [string]$LogDir = "",
    [switch]$Protected,
    [switch]$KeepTemp,
    [switch]$CaptureGuestConsole,
    [switch]$CaptureCrosvmStdio,
    [switch]$DryRun,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$VmArgs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptRepoRoot = $RepoRoot
if (-not $scriptRepoRoot) {
    $scriptRepoRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
}

$wrapper = Join-Path $PSScriptRoot "vm_windows.ps1"

if ($VmArgs) {
    & $wrapper `
        -Command "run-microdroid" `
        -RepoRoot $scriptRepoRoot `
        -WorkDir $WorkDir `
        -LogDir $LogDir `
        -Protected:$Protected `
        -KeepTemp:$KeepTemp `
        -CaptureGuestConsole:$CaptureGuestConsole `
        -CaptureCrosvmStdio:$CaptureCrosvmStdio `
        -DryRun:$DryRun `
        @VmArgs
} else {
    & $wrapper `
        -Command "run-microdroid" `
        -RepoRoot $scriptRepoRoot `
        -WorkDir $WorkDir `
        -LogDir $LogDir `
        -Protected:$Protected `
        -KeepTemp:$KeepTemp `
        -CaptureGuestConsole:$CaptureGuestConsole `
        -CaptureCrosvmStdio:$CaptureCrosvmStdio `
        -DryRun:$DryRun
}

exit $LASTEXITCODE
