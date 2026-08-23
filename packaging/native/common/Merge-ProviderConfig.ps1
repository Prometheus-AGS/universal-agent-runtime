[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Config,
    [Parameter(Mandatory = $true)]
    [string]$EnvFile,
    [Parameter(Mandatory = $true)]
    [string]$ProxyUrl
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $Config -PathType Leaf) -or
    -not (Test-Path -LiteralPath $EnvFile -PathType Leaf)) {
    throw 'config and service environment files must exist'
}

function Find-SectionStart {
    param([string[]]$Lines, [string]$Name)
    for ($index = 0; $index -lt $Lines.Count; $index++) {
        if ($Lines[$index] -match ('^' + [regex]::Escape($Name) + '\s*:')) {
            return $index
        }
    }
    return -1
}

function Find-SectionEnd {
    param([string[]]$Lines, [int]$Start)
    for ($index = $Start + 1; $index -lt $Lines.Count; $index++) {
        if ($Lines[$index] -match '^[^\s#]') {
            return $index
        }
    }
    return $Lines.Count
}

function Insert-Lines {
    param([string[]]$Lines, [int]$Index, [string[]]$Additions)
    $result = [System.Collections.Generic.List[string]]::new()
    for ($position = 0; $position -lt $Index; $position++) {
        $result.Add($Lines[$position])
    }
    foreach ($line in $Additions) {
        $result.Add($line)
    }
    for ($position = $Index; $position -lt $Lines.Count; $position++) {
        $result.Add($Lines[$position])
    }
    return $result.ToArray()
}

function New-ModelLines {
    param(
        [string]$Id,
        [string]$DisplayName,
        [Nullable[int64]]$ContextWindow,
        [Nullable[int64]]$MaxOutput,
        [bool]$Vision,
        [bool]$Tools,
        [bool]$Reasoning,
        [bool]$Structured
    )
    $lines = [System.Collections.Generic.List[string]]::new()
    $lines.Add('      - id: ' + ($Id | ConvertTo-Json -Compress))
    $lines.Add('        display_name: ' + ($DisplayName | ConvertTo-Json -Compress))
    if ($null -ne $ContextWindow) { $lines.Add("        context_window: $ContextWindow") }
    $lines.Add('        supports_vision: ' + $Vision.ToString().ToLowerInvariant())
    $lines.Add('        supports_tools: ' + $Tools.ToString().ToLowerInvariant())
    $lines.Add('        supports_reasoning: ' + $Reasoning.ToString().ToLowerInvariant())
    $lines.Add('        supports_structured_output: ' + $Structured.ToString().ToLowerInvariant())
    $lines.Add('        supports_streaming: true')
    if ($null -ne $MaxOutput) { $lines.Add("        max_output_tokens: $MaxOutput") }
    $lines.Add('        enabled: true')
    return $lines.ToArray()
}

function New-ProviderLines {
    param(
        [string]$Id,
        [string]$DisplayName,
        [string]$BaseUrl,
        [string]$DefaultModel,
        [string[]]$Models
    )
    $lines = [System.Collections.Generic.List[string]]::new()
    $lines.Add('  - id: ' + ($Id | ConvertTo-Json -Compress))
    $lines.Add('    display_name: ' + ($DisplayName | ConvertTo-Json -Compress))
    $lines.Add('    base_url: ' + ($BaseUrl | ConvertTo-Json -Compress))
    $lines.Add('    protocol: chat')
    $lines.Add('    default_model: ' + ($DefaultModel | ConvertTo-Json -Compress))
    $lines.Add('    enabled: true')
    $lines.Add('    models:')
    foreach ($line in $Models) { $lines.Add($line) }
    return $lines.ToArray()
}

$present = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
foreach ($line in [IO.File]::ReadAllLines($EnvFile)) {
    if ($line -match '^(KIMI_API_KEY|MINIMAX_API_KEY|DASHSCOPE_API_KEY|MOONSHOT_API_KEY|ZAI_API_KEY)=(.+)$') {
        [void]$present.Add($Matches[1])
    }
}

$proxyModels = @()
try {
    $inventory = Invoke-RestMethod -Method Get -Uri ($ProxyUrl.TrimEnd('/') + '/models') -TimeoutSec 5
    $proxyModels = @($inventory.data | ForEach-Object { $_.id } |
        Where-Object { $_ -is [string] -and $_ -match '^[A-Za-z0-9._:/+\-]+$' } |
        Sort-Object -Unique)
} catch {
    Write-Warning ('local proxy inventory unavailable; proxy seed omitted: ' + $_.Exception.GetType().Name)
}

$providers = [ordered]@{}
if ($proxyModels.Count -gt 0) {
    $models = [System.Collections.Generic.List[string]]::new()
    foreach ($modelId in $proxyModels) {
        foreach ($line in (New-ModelLines -Id $modelId -DisplayName $modelId -ContextWindow $null -MaxOutput $null -Vision $false -Tools $true -Reasoning $false -Structured $false)) {
            $models.Add($line)
        }
    }
    $providers['local-openai-proxy'] = New-ProviderLines -Id 'local-openai-proxy' -DisplayName 'Local OpenAI Proxy' -BaseUrl $ProxyUrl.TrimEnd('/') -DefaultModel $proxyModels[0] -Models $models.ToArray()
}
if ($present.Contains('KIMI_API_KEY')) {
    $providers['kimi-for-coding'] = New-ProviderLines -Id 'kimi-for-coding' -DisplayName 'Kimi For Coding' -BaseUrl 'https://api.kimi.com/coding/v1' -DefaultModel 'k3' -Models (New-ModelLines -Id 'k3' -DisplayName 'Kimi K3' -ContextWindow 1048576 -MaxOutput 131072 -Vision $true -Tools $true -Reasoning $true -Structured $true)
}
if ($present.Contains('MINIMAX_API_KEY')) {
    $providers['minimax'] = New-ProviderLines -Id 'minimax' -DisplayName 'MiniMax' -BaseUrl 'https://api.minimax.io/v1' -DefaultModel 'MiniMax-M3' -Models (New-ModelLines -Id 'MiniMax-M3' -DisplayName 'MiniMax M3' -ContextWindow 1000000 -MaxOutput 128000 -Vision $true -Tools $true -Reasoning $true -Structured $false)
}
if ($present.Contains('DASHSCOPE_API_KEY')) {
    $providers['alibaba'] = New-ProviderLines -Id 'alibaba' -DisplayName 'Alibaba/Qwen' -BaseUrl 'https://dashscope-intl.aliyuncs.com/compatible-mode/v1' -DefaultModel 'qwen3-coder-plus' -Models (New-ModelLines -Id 'qwen3-coder-plus' -DisplayName 'Qwen3 Coder Plus' -ContextWindow 1048576 -MaxOutput 65536 -Vision $false -Tools $true -Reasoning $false -Structured $false)
}
if ($present.Contains('ZAI_API_KEY')) {
    $zaiModels = (New-ModelLines -Id 'glm-4.7' -DisplayName 'GLM-4.7' -ContextWindow 204800 -MaxOutput 131072 -Vision $false -Tools $true -Reasoning $true -Structured $false) +
        (New-ModelLines -Id 'glm-5.2' -DisplayName 'GLM-5.2' -ContextWindow 1000000 -MaxOutput 131072 -Vision $false -Tools $true -Reasoning $true -Structured $true)
    $providers['zai'] = New-ProviderLines -Id 'zai' -DisplayName 'Z.AI' -BaseUrl 'https://api.z.ai/api/paas/v4' -DefaultModel 'glm-5.2' -Models $zaiModels
}
if ($present.Contains('MOONSHOT_API_KEY')) {
    $moonshotModels = (New-ModelLines -Id 'kimi-k2.5' -DisplayName 'Kimi K2.5' -ContextWindow 262144 -MaxOutput 262144 -Vision $true -Tools $true -Reasoning $true -Structured $true) +
        (New-ModelLines -Id 'kimi-k3' -DisplayName 'Kimi K3' -ContextWindow 1048576 -MaxOutput 131072 -Vision $true -Tools $true -Reasoning $true -Structured $true)
    $providers['moonshotai'] = New-ProviderLines -Id 'moonshotai' -DisplayName 'Moonshot AI' -BaseUrl 'https://api.moonshot.ai/v1' -DefaultModel 'kimi-k2.5' -Models $moonshotModels
}

$lines = [IO.File]::ReadAllLines($Config)
$serverStart = Find-SectionStart -Lines $lines -Name 'server'
if ($serverStart -lt 0) {
    $lines = Insert-Lines -Lines $lines -Index $lines.Count -Additions @('', 'server:', '  host: "127.0.0.1"', '  port: 1906', '  grpc_port: 50051')
} else {
    if ($lines[$serverStart].Trim() -ne 'server:') { throw 'top-level server must be a block mapping' }
    $serverEnd = Find-SectionEnd -Lines $lines -Start $serverStart
    $serverText = ($lines[($serverStart + 1)..($serverEnd - 1)] -join "`n")
    $missing = [System.Collections.Generic.List[string]]::new()
    if ($serverText -notmatch '(?m)^  host\s*:') { $missing.Add('  host: "127.0.0.1"') }
    if ($serverText -notmatch '(?m)^  port\s*:') { $missing.Add('  port: 1906') }
    if ($serverText -notmatch '(?m)^  grpc_port\s*:') { $missing.Add('  grpc_port: 50051') }
    $lines = Insert-Lines -Lines $lines -Index $serverEnd -Additions $missing.ToArray()
}

if ($providers.Count -gt 0) {
    $providerStart = Find-SectionStart -Lines $lines -Name 'providers'
    if ($providerStart -lt 0) {
        $lines = Insert-Lines -Lines $lines -Index $lines.Count -Additions @('', 'providers:')
        $providerStart = $lines.Count - 1
    } elseif (($lines[$providerStart] -split ':', 2)[1].Trim() -eq '[]') {
        $lines[$providerStart] = 'providers:'
    } elseif (-not [string]::IsNullOrWhiteSpace(($lines[$providerStart] -split ':', 2)[1])) {
        throw 'top-level providers must be a block sequence or []'
    }
    $providerEnd = Find-SectionEnd -Lines $lines -Start $providerStart
    $existing = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($line in $lines[($providerStart + 1)..($providerEnd - 1)]) {
        if ($line -match '^  - id:\s*["'']?([^"''#\s]+)') { [void]$existing.Add($Matches[1]) }
    }
    $additions = [System.Collections.Generic.List[string]]::new()
    foreach ($entry in $providers.GetEnumerator()) {
        if (-not $existing.Contains($entry.Key)) {
            foreach ($line in $entry.Value) { $additions.Add($line) }
        }
    }
    $lines = Insert-Lines -Lines $lines -Index $providerEnd -Additions $additions.ToArray()
}

$temporary = Join-Path (Split-Path -Parent $Config) ('.uar-config.' + [guid]::NewGuid().ToString('N') + '.tmp')
try {
    [IO.File]::WriteAllLines($temporary, $lines, [Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath $temporary -Destination $Config -Force
} finally {
    Remove-Item -LiteralPath $temporary -Force -ErrorAction SilentlyContinue
}
