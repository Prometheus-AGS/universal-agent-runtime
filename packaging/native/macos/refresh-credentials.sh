#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 || ! -f "$1" ]]; then
  echo "usage: $0 <complete-service-env-file>" >&2
  exit 2
fi

destination="$HOME/.uar/service.env"
mkdir -p "$(dirname "$destination")"
temporary="$destination.$$.tmp"
install -m 600 "$1" "$temporary"
mv -f "$temporary" "$destination"

script_dir=$(cd "$(dirname "$0")" && pwd)
"$script_dir/control.sh" restart
echo "credentials refreshed"
