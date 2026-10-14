# Run deliberately as administrator after platform qualification. This does not
# install Python or credentials and never reboots the host.
param(
    [Parameter(Mandatory=$true)][string]$PythonPath,
    [Parameter(Mandatory=$true)][string]$ControllerPath,
    [Parameter(Mandatory=$true)][string]$ConfigPath
)
$ErrorActionPreference = 'Stop'
foreach ($value in @($PythonPath, $ControllerPath, $ConfigPath)) {
    if (-not [System.IO.Path]::IsPathRooted($value) -or -not (Test-Path -LiteralPath $value -PathType Leaf)) {
        throw 'Absolute installed Python, controller and private configuration paths are required.'
    }
    if ($value.Contains('"')) { throw 'Quoted path segments are unsupported.' }
}
$commandLine = '"' + $PythonPath + '" "' + $ControllerPath + '" --config "' + $ConfigPath + '" --windows-service'
New-Service -Name 'UARSupervisor' -DisplayName 'UAR trusted supervisor exclusion controller' -BinaryPathName $commandLine -StartupType Automatic
