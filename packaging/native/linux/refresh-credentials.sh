#!/usr/bin/env bash
set -euo pipefail

if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
  echo "credential refresh must run as root" >&2
  exit 1
fi
if [[ $# -ne 0 ]]; then
  echo "usage: sudo --preserve-env=KIMI_API_KEY,KIMI_CODING_API_KEY,KIMI_CODING_KEY,MINIMAX_API_KEY,MINIMAX_KEY,DASHSCOPE_API_KEY,QWEN_API_KEY,QWEN_TOKEN_PLAN_API_KEY,MOONSHOT_API_KEY,ZAI_API_KEY $0" >&2
  exit 2
fi

destination="/etc/uar/uar.env"
config_path="/etc/uar/config.yaml"
script_dir=$(cd "$(dirname "$0")" && pwd)
"$script_dir/../common/generate-provider-env.sh" --output "$destination"
"$script_dir/../common/merge-provider-config.sh" \
  --config "$config_path" \
  --env-file "$destination" \
  --proxy-url "http://127.0.0.1:8181/v1"
chown root:uar "$destination" "$config_path"
chmod 640 "$destination" "$config_path"
systemctl restart uar.service
echo "credentials refreshed"
