#!/usr/bin/env bash
set -euo pipefail

case "${1-}" in
  start|stop|restart|status)
    systemctl "$1" uar.service
    ;;
  *)
    echo "usage: $0 {start|stop|restart|status}" >&2
    exit 2
    ;;
esac
