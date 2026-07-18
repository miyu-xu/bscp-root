param(
    [Parameter(Mandatory = $true)][string]$PipeOut,
    [Parameter(Mandatory = $true)][string]$PipeIn,
    [Parameter(Mandatory = $true)][int]$TcpPort,
    [int]$BufferSize = 65536,
    [string]$TcpHost = "127.0.0.1"
)

$ErrorActionPreference = "Stop"
$bridge = Join-Path $PSScriptRoot "cf_hvc_bridge.py"
if (-not (Test-Path $bridge)) {
    throw "Missing cross-platform HVC bridge: $bridge"
}

& python $bridge `
    --guest-out $PipeOut `
    --guest-in $PipeIn `
    --tcp-host $TcpHost `
    --tcp-port $TcpPort `
    --reconnect
exit $LASTEXITCODE
