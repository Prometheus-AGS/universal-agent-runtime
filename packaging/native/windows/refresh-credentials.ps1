[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$serviceName = 'PrometheusUniversalAgentRuntime'
$configRoot = Join-Path $env:ProgramData 'Prometheus\UniversalAgentRuntime\config'
$destination = Join-Path $configRoot 'uar.env'
$configPath = Join-Path $configRoot 'config.yaml'
$environmentGenerator = Join-Path $PSScriptRoot '..\common\Generate-ProviderEnv.ps1'
$configMerger = Join-Path $PSScriptRoot '..\common\Merge-ProviderConfig.ps1'
New-Item -ItemType Directory -Force -Path $configRoot | Out-Null
& $environmentGenerator -Output $destination
& $configMerger -Config $configPath -EnvFile $destination -ProxyUrl 'http://127.0.0.1:8181/v1'
& icacls.exe $destination /inheritance:r /grant:r '*S-1-5-18:(F)' '*S-1-5-32-544:(F)' '*S-1-5-19:(R)' | Out-Null
Restart-Service -Name $serviceName
Write-Output 'credentials refreshed'
