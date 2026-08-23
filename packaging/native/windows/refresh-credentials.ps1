[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Source
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) {
    throw "complete service environment file not found: $Source"
}

$serviceName = 'PrometheusUniversalAgentRuntime'
$configRoot = Join-Path $env:ProgramData 'Prometheus\UniversalAgentRuntime\config'
$destination = Join-Path $configRoot 'uar.env'
$temporary = "$destination.$PID.tmp"
New-Item -ItemType Directory -Force -Path $configRoot | Out-Null
Copy-Item -LiteralPath $Source -Destination $temporary -Force
& icacls.exe $temporary /inheritance:r /grant:r '*S-1-5-18:(F)' '*S-1-5-32-544:(F)' '*S-1-5-19:(R)' | Out-Null
Move-Item -LiteralPath $temporary -Destination $destination -Force
Restart-Service -Name $serviceName
Write-Output 'credentials refreshed'
