[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$serviceName = 'PrometheusUniversalAgentRuntime'
$programRoot = Join-Path $env:ProgramFiles 'Prometheus\UniversalAgentRuntime'
$service = Get-Service -Name $serviceName -ErrorAction SilentlyContinue

if ($null -ne $service) {
    if ($service.Status -ne 'Stopped') {
        Stop-Service -Name $serviceName -Force
        $service.WaitForStatus('Stopped', [TimeSpan]::FromSeconds(35))
    }
    & sc.exe delete $serviceName | Out-Null
}
if (Test-Path -LiteralPath $programRoot) {
    Remove-Item -LiteralPath $programRoot -Recurse -Force
}

Write-Output 'uninstalled service and program files; ProgramData configuration, database state, backups, and logs were preserved'
