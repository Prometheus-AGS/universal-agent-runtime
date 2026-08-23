[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Binary,
    [Parameter(Mandatory = $true)]
    [string]$StaticDir,
    [string]$Config
)

$ErrorActionPreference = 'Stop'
$serviceName = 'PrometheusUniversalAgentRuntime'
$programRoot = Join-Path $env:ProgramFiles 'Prometheus\UniversalAgentRuntime'
$dataRoot = Join-Path $env:ProgramData 'Prometheus\UniversalAgentRuntime'
$configRoot = Join-Path $dataRoot 'config'
$configPath = Join-Path $configRoot 'config.yaml'
$environmentPath = Join-Path $configRoot 'uar.env'
$logRoot = Join-Path $dataRoot '.prometheus\logs'
$backupRoot = Join-Path $dataRoot '.prometheus\backups'
$stateRoot = Join-Path $dataRoot 'state'
$binaryPath = Join-Path $programRoot 'universal-agent-runtime.exe'
$staticPath = Join-Path $programRoot 'static'

if (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) {
    throw "release binary not found: $Binary"
}
if (-not (Test-Path -LiteralPath $StaticDir -PathType Container)) {
    throw "React bundle not found: $StaticDir"
}
if (-not (Test-Path -LiteralPath $configPath -PathType Leaf) -and
    ([string]::IsNullOrWhiteSpace($Config) -or -not (Test-Path -LiteralPath $Config -PathType Leaf))) {
    throw 'first install requires -Config <initial-config>'
}

New-Item -ItemType Directory -Force -Path $programRoot, $configRoot, $logRoot, $backupRoot, $stateRoot | Out-Null

$existingService = Get-Service -Name $serviceName -ErrorAction SilentlyContinue
if ($null -ne $existingService -and $existingService.Status -ne 'Stopped') {
    Stop-Service -Name $serviceName -Force
    $existingService.WaitForStatus('Stopped', [TimeSpan]::FromSeconds(35))
}

$timestamp = (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ')
if (Test-Path -LiteralPath $configPath -PathType Leaf) {
    Copy-Item -LiteralPath $configPath -Destination (Join-Path $backupRoot "config.yaml.$timestamp")
} elseif (-not [string]::IsNullOrWhiteSpace($Config)) {
    Copy-Item -LiteralPath $Config -Destination $configPath
}

Copy-Item -LiteralPath $Binary -Destination $binaryPath -Force
if (Test-Path -LiteralPath $staticPath) {
    Remove-Item -LiteralPath $staticPath -Recurse -Force
}
Copy-Item -LiteralPath $StaticDir -Destination $staticPath -Recurse

if (-not (Test-Path -LiteralPath $environmentPath -PathType Leaf)) {
    $logFile = (Join-Path $logRoot 'operational.log').Replace('\', '/')
    @(
        'UAR_SERVER__HOST=127.0.0.1'
        'UAR_SERVER__GRPC_PORT=50051'
        'PORT=1906'
        "UAR_LOG_FILE=$logFile"
    ) | Set-Content -LiteralPath $environmentPath -Encoding utf8NoBOM
}

& icacls.exe $configRoot /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)(F)' '*S-1-5-32-544:(OI)(CI)(F)' '*S-1-5-19:(OI)(CI)(RX)' | Out-Null
& icacls.exe $logRoot /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)(F)' '*S-1-5-32-544:(OI)(CI)(F)' '*S-1-5-19:(OI)(CI)(M)' | Out-Null
& icacls.exe $stateRoot /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)(F)' '*S-1-5-32-544:(OI)(CI)(F)' '*S-1-5-19:(OI)(CI)(M)' | Out-Null

$serviceCommand = ('"{0}" --config "{1}" --env-file "{2}" --port 1906 service' -f $binaryPath, $configPath, $environmentPath)
if ($null -eq $existingService) {
    New-Service -Name $serviceName -BinaryPathName $serviceCommand -DisplayName 'Prometheus Universal Agent Runtime' -StartupType Automatic | Out-Null
} else {
    & sc.exe config $serviceName binPath= $serviceCommand start= auto | Out-Null
}
& sc.exe config $serviceName obj= 'NT AUTHORITY\LocalService' password= '' | Out-Null
& sc.exe failure $serviceName reset= 86400 actions= restart/5000 | Out-Null
Start-Service -Name $serviceName
Write-Output "installed $serviceName"
