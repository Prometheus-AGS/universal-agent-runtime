#!/usr/bin/env bash
set -euo pipefail

if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
  echo "uninstall must run as root" >&2
  exit 1
fi

systemctl disable --now uar.service >/dev/null 2>&1 || true
rm -f /etc/systemd/system/uar.service
systemctl daemon-reload
rm -rf /usr/local/lib/uar

echo "uninstalled uar.service; /etc/uar and /var/lib/uar were preserved"
