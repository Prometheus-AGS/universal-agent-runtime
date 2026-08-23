[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('start', 'stop', 'restart', 'status')]
    [string]$Action
)

$ErrorActionPreference = 'Stop'
$serviceName = 'PrometheusUniversalAgentRuntime'

switch ($Action) {
    'start' { Start-Service -Name $serviceName }
    'stop' { Stop-Service -Name $serviceName }
    'restart' { Restart-Service -Name $serviceName }
    'status' { Get-Service -Name $serviceName }
}
