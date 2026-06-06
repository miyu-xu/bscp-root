# Obtain OVMF_DEBUG.fd: local build (WSL), copy from path, or GitHub Actions artifact.
param(
    [string]$LocalPath = "",
    [string]$WorkflowRunId = "",
    [string]$Repo = "",
    [switch]$Force,
    [switch]$Build
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path "$PSScriptRoot\..").Path
$FwDir = Join-Path $RepoRoot "out\dist\firmware"
$OutFd = Join-Path $FwDir "OVMF_DEBUG.fd"
New-Item -ItemType Directory -Force -Path $FwDir | Out-Null

if ((Test-Path $OutFd) -and -not $Force -and -not $LocalPath -and -not $Build) {
    Write-Host "OK: $OutFd ($((Get-Item $OutFd).Length) bytes)"
    exit 0
}

if ($LocalPath) {
    if (-not (Test-Path $LocalPath)) { throw "Not found: $LocalPath" }
    Copy-Item $LocalPath $OutFd -Force
    Write-Host "Copied to $OutFd"
    exit 0
}

if ($Build) {
    & (Join-Path $PSScriptRoot "build_ovmf_debug.ps1") -Force:$Force
    exit $LASTEXITCODE
}

if ($WorkflowRunId) {
    $gh = Get-Command gh -ErrorAction SilentlyContinue
    if (-not $gh) { throw "gh CLI required for artifact download. Install GitHub CLI or use -Build." }
    if (-not $Repo) {
        $remote = git -C $RepoRoot remote get-url origin 2>$null
        if ($remote -match 'github\.com[:/](.+?)(?:\.git)?$') { $Repo = $Matches[1] }
    }
    if (-not $Repo) { throw "Set -Repo owner/name for artifact download" }
    $dest = Join-Path $env:TEMP "ovmf-debug-artifact"
    if (Test-Path $dest) { Remove-Item $dest -Recurse -Force }
    & gh run download $WorkflowRunId -R $Repo -n "OVMF_DEBUG.fd" -D $dest
    $artifact = Get-ChildItem $dest -Recurse -Filter "OVMF_DEBUG.fd" | Select-Object -First 1
    if (-not $artifact) { throw "Artifact OVMF_DEBUG.fd not in download" }
    Copy-Item $artifact.FullName $OutFd -Force
    Write-Host "Downloaded to $OutFd"
    exit 0
}

Write-Host @"
OVMF_DEBUG.fd not found. Options:
  1) .\scripts\fetch_ovmf_debug.ps1 -Build
  2) .\scripts\build_ovmf_debug.ps1
  3) GitHub Actions: run workflow 'Build OVMF DEBUG', then:
     .\scripts\fetch_ovmf_debug.ps1 -WorkflowRunId <id> -Repo owner/repo
  4) Copy manually to out\dist\firmware\OVMF_DEBUG.fd
"@
exit 1
