[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Binary,
    [Parameter(Mandatory = $true)]
    [string]$StaticDir,
    [string]$Config
)

$arguments = @('-Binary', $Binary, '-StaticDir', $StaticDir)
if (-not [string]::IsNullOrWhiteSpace($Config)) {
    $arguments += @('-Config', $Config)
}
& (Join-Path $PSScriptRoot 'install.ps1') @arguments
