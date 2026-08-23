[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Output
)

$ErrorActionPreference = 'Stop'
$providerNames = @(
    'KIMI_API_KEY',
    'KIMI_CODING_API_KEY',
    'KIMI_CODING_KEY',
    'MINIMAX_API_KEY',
    'MINIMAX_KEY',
    'DASHSCOPE_API_KEY',
    'QWEN_API_KEY',
    'QWEN_TOKEN_PLAN_API_KEY',
    'MOONSHOT_API_KEY',
    'ZAI_API_KEY'
)

function Resolve-ProviderValue {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Names
    )

    foreach ($name in $Names) {
        $value = [Environment]::GetEnvironmentVariable($name)
        if (-not [string]::IsNullOrEmpty($value)) {
            if ($value.Contains("`r") -or $value.Contains("`n")) {
                throw "provider credential $($Names[0]) contains a line break"
            }
            return $value
        }
    }
    return $null
}

function ConvertTo-DotEnvValue {
    param([Parameter(Mandatory = $true)][string]$Value)

    $escaped = $Value.Replace('\', '\\').Replace('"', '\"').Replace('$', '\$')
    return '"' + $escaped + '"'
}

$outputDirectory = Split-Path -Parent $Output
if ([string]::IsNullOrWhiteSpace($outputDirectory)) {
    $outputDirectory = (Get-Location).Path
}
New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
$temporary = Join-Path $outputDirectory ('.uar-env.' + [guid]::NewGuid().ToString('N') + '.tmp')

try {
    $lines = [System.Collections.Generic.List[string]]::new()
    if (Test-Path -LiteralPath $Output -PathType Leaf) {
        foreach ($line in [IO.File]::ReadAllLines($Output)) {
            $name = if ($line -match '^([A-Z][A-Z0-9_]*)=') { $Matches[1] } else { $null }
            if ($name -notin $providerNames) {
                $lines.Add($line)
            }
        }
    }

    $mappings = @(
        @('KIMI_API_KEY', 'KIMI_CODING_API_KEY', 'KIMI_CODING_KEY'),
        @('MINIMAX_API_KEY', 'MINIMAX_KEY'),
        @('DASHSCOPE_API_KEY', 'QWEN_API_KEY', 'QWEN_TOKEN_PLAN_API_KEY'),
        @('MOONSHOT_API_KEY'),
        @('ZAI_API_KEY')
    )
    foreach ($mapping in $mappings) {
        $value = Resolve-ProviderValue -Names $mapping
        if ($null -ne $value) {
            $lines.Add("$($mapping[0])=$(ConvertTo-DotEnvValue -Value $value)")
        }
    }

    [IO.File]::WriteAllLines($temporary, $lines, [Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath $temporary -Destination $Output -Force
} finally {
    Remove-Item -LiteralPath $temporary -Force -ErrorAction SilentlyContinue
}
