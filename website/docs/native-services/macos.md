---
sidebar_position: 2
title: macOS LaunchAgent
description: Build, install, operate, upgrade, and remove the UAR user LaunchAgent.
---

# macOS LaunchAgent

## Install

Prerequisites are Rust, Node.js, pnpm 11.15.0, Python 3, and the provider
variables needed by this deployment. From the repository root:

```bash
pnpm install --frozen-lockfile
pnpm build
cargo build --locked --release --no-default-features --features server-full
source "$HOME/.bash_profile"
packaging/native/macos/install.sh \
  --binary target/release/universal-agent-runtime \
  --static-dir static
```

The user LaunchAgent label is `com.prometheus.universal-agent-runtime`. Open
`http://localhost:1906` after both probes succeed:

```bash
packaging/native/macos/control.sh status
curl --fail http://localhost:1906/healthz
curl --fail http://localhost:1906/readyz
```

## Files and exposure

| Purpose | Path |
|---|---|
| binary | `~/.uar/bin/universal-agent-runtime` |
| React bundle | `~/.uar/static/` |
| YAML | `~/.uar/config.yaml` |
| service environment | `~/.uar/service.env` |
| database state | `~/.uar/data/` |
| LaunchAgent | `~/Library/LaunchAgents/com.prometheus.universal-agent-runtime.plist` |
| logs | `~/.prometheus/logs/universal-agent-runtime/` |
| config backups | `~/.prometheus/backups/uar/` |

HTTP `1906` and gRPC `50051` inherit `server.host`, which the native default
sets to `127.0.0.1`. Do not change it to a wildcard without adding production
authentication and network policy.

## Operate, refresh, upgrade, uninstall

```bash
packaging/native/macos/control.sh start|stop|restart|status
source "$HOME/.bash_profile"
packaging/native/macos/refresh-credentials.sh
packaging/native/macos/upgrade.sh --binary target/release/universal-agent-runtime --static-dir static
packaging/native/macos/uninstall.sh
```

Refresh imports only approved provider variables and performs one restart.
Upgrade preserves YAML, database state, selected model, and API/UI provider
settings except for the exact native migration from `alibaba/qwen3.7-max` to
released `alibaba/qwen3.8-max` and its malformed legacy credential reference.
Uninstall removes the LaunchAgent, binary, and static bundle while
retaining configuration, state, environment, backups, and logs.

## Troubleshoot and verify

Use `launchctl print gui/$UID/com.prometheus.universal-agent-runtime` and inspect
the `.prometheus` log directory. An unreadable `--env-file`, malformed dotenv
file, or unusable `UAR_LOG_FILE` fails startup before a listener opens. Verify
provider/model visibility in **Admin → Providers** and **Admin → Models**; then
send a short genuine inference request. Retained runtime evidence for this
release is macOS/server-full only.
