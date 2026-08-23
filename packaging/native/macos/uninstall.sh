#!/usr/bin/env bash
set -euo pipefail

service_label="com.prometheus.universal-agent-runtime"
plist_path="$HOME/Library/LaunchAgents/$service_label.plist"

launchctl bootout "gui/$UID" "$plist_path" >/dev/null 2>&1 || true
rm -f "$plist_path"
rm -f "$HOME/.uar/bin/universal-agent-runtime"
rm -rf "$HOME/.uar/static"

echo "uninstalled $service_label; configuration, service.env, database state, backups, and logs were preserved"
