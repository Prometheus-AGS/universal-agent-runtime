#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 0 ]]; then
  echo "usage: $0" >&2
  exit 2
fi

destination="$HOME/.uar/service.env"
script_dir=$(cd "$(dirname "$0")" && pwd)
config_path="$HOME/.uar/config.yaml"
"$script_dir/../common/generate-provider-env.sh" --output "$destination"
"$script_dir/../common/merge-provider-config.sh" \
  --config "$config_path" \
  --env-file "$destination" \
  --proxy-url "http://127.0.0.1:8181/v1"
"$script_dir/control.sh" restart
echo "credentials refreshed"
