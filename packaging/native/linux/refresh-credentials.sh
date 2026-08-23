#!/usr/bin/env bash
set -euo pipefail

if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
  echo "credential refresh must run as root" >&2
  exit 1
fi
if [[ $# -ne 1 || ! -f "$1" ]]; then
  echo "usage: sudo $0 <complete-service-env-file>" >&2
  exit 2
fi

destination="/etc/uar/uar.env"
temporary="$destination.$$.tmp"
install -m 640 -o root -g uar "$1" "$temporary"
mv -f "$temporary" "$destination"
systemctl restart uar.service
echo "credentials refreshed"
