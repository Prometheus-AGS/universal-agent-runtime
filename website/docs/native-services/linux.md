---
sidebar_position: 3
title: Linux systemd
description: Install and operate the UAR systemd service with platform-owned paths.
---

# Linux systemd

## Install

Build or transfer the release binary and React `static/` bundle to the target.
Python 3 is required by additive YAML bootstrap. From a shell containing the
approved provider variables:

```bash
sudo --preserve-env=KIMI_API_KEY,KIMI_CODING_API_KEY,KIMI_CODING_KEY,MINIMAX_API_KEY,MINIMAX_KEY,DASHSCOPE_API_KEY,QWEN_API_KEY,QWEN_TOKEN_PLAN_API_KEY,MOONSHOT_API_KEY,ZAI_API_KEY \
  packaging/native/linux/install.sh \
  --binary target/release/universal-agent-runtime \
  --static-dir static
sudo packaging/native/linux/control.sh status
```

`uar.service` uses direct `ExecStart`, `WorkingDirectory=/var/lib/uar`,
`EnvironmentFile=/etc/uar/uar.env`, `SIGTERM`, and `Restart=on-failure`.
When Alibaba credentials are present, bootstrap seeds released
`alibaba/qwen3.8-max`; it does not seed the retired preview.

| Purpose | Path |
|---|---|
| program and React bundle | `/usr/local/lib/uar/` |
| YAML and environment | `/etc/uar/` |
| database/state | `/var/lib/uar/` |
| logs | `/var/lib/uar/.prometheus/logs/` |
| backups | `/var/lib/uar/.prometheus/backups/` |

## Lifecycle

```bash
sudo packaging/native/linux/control.sh start|stop|restart|status
sudo --preserve-env=KIMI_API_KEY,MINIMAX_API_KEY,DASHSCOPE_API_KEY,MOONSHOT_API_KEY,ZAI_API_KEY \
  packaging/native/linux/refresh-credentials.sh
sudo packaging/native/linux/upgrade.sh --binary target/release/universal-agent-runtime --static-dir static
sudo packaging/native/linux/uninstall.sh
```

Uninstall removes the unit and program directory but preserves `/etc/uar` and
`/var/lib/uar`. Confirm both ports listen only on loopback, inspect the
`.prometheus` logs, and complete genuine inference on the Linux host before a
Linux runtime claim. The macOS gate establishes unit structure and cross-target
compilation only.
