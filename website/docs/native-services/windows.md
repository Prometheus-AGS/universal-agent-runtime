---
sidebar_position: 4
title: Windows service
description: Install UAR through native Windows SCM integration without a wrapper daemon.
---

# Windows service

## Install

Obtain or build `universal-agent-runtime.exe` for MSVC, build the React bundle,
and open an elevated PowerShell session with approved provider variables set:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\packaging\native\windows\install.ps1 `
  -Binary .\target\release\universal-agent-runtime.exe `
  -StaticDir .\static
.\packaging\native\windows\control.ps1 status
```

The installer registers `PrometheusUniversalAgentRuntime` directly with SCM.
The process runs as `NT AUTHORITY\LocalService`; SCM Stop and Shutdown controls
enter UAR's graceful cancellation path.

| Purpose | Path |
|---|---|
| program and React bundle | `%ProgramFiles%\Prometheus\UniversalAgentRuntime\` |
| YAML and environment | `%ProgramData%\Prometheus\UniversalAgentRuntime\config\` |
| database/state | `%ProgramData%\Prometheus\UniversalAgentRuntime\state\` |
| logs | `%ProgramData%\Prometheus\UniversalAgentRuntime\.prometheus\logs\` |
| backups | `%ProgramData%\Prometheus\UniversalAgentRuntime\.prometheus\backups\` |

## Lifecycle

```powershell
.\packaging\native\windows\control.ps1 start|stop|restart|status
.\packaging\native\windows\refresh-credentials.ps1
.\packaging\native\windows\upgrade.ps1 -Binary .\uar.exe -StaticDir .\static
.\packaging\native\windows\uninstall.ps1
```

Equivalent `.cmd` wrappers accompany each entrypoint. Credential refresh copies
only approved canonical provider variables from the current process and
restarts once. Alibaba bootstrap seeds released `alibaba/qwen3.8-max` and not
the retired preview. Upgrade preserves configuration and state; uninstall removes SCM
registration and Program Files while retaining ProgramData.

Check `Get-Service PrometheusUniversalAgentRuntime`, the `.prometheus` logs,
`http://localhost:1906/healthz`, and both loopback listeners. Complete genuine
provider inference on Windows before making a Windows runtime claim. The macOS
gate proves only MSVC compilation and PowerShell/template structure.
