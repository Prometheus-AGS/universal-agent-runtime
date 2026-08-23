#!/usr/bin/env bash
set -euo pipefail

service_label="com.prometheus.universal-agent-runtime"
plist_path="$HOME/Library/LaunchAgents/$service_label.plist"
domain="gui/$UID"

case "${1-}" in
  start)
    launchctl bootstrap "$domain" "$plist_path" 2>/dev/null || true
    launchctl kickstart -k "$domain/$service_label"
    ;;
  stop)
    launchctl bootout "$domain" "$plist_path"
    ;;
  restart)
    launchctl bootout "$domain" "$plist_path" >/dev/null 2>&1 || true
    launchctl bootstrap "$domain" "$plist_path"
    launchctl kickstart -k "$domain/$service_label"
    ;;
  status)
    launchctl print "$domain/$service_label"
    ;;
  *)
    echo "usage: $0 {start|stop|restart|status}" >&2
    exit 2
    ;;
esac
