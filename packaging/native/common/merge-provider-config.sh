#!/usr/bin/env bash
set -euo pipefail

config_path=""
environment_path=""
proxy_url=""

usage() {
  echo "usage: $0 --config <config-yaml> --env-file <service-env-file> --proxy-url <openai-base-url>" >&2
}

while (($#)); do
  case "$1" in
    --config)
      config_path=${2-}
      shift 2
      ;;
    --env-file)
      environment_path=${2-}
      shift 2
      ;;
    --proxy-url)
      proxy_url=${2-}
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      exit 2
      ;;
  esac
done

if [[ -z "$config_path" || -z "$environment_path" || -z "$proxy_url" ]]; then
  usage
  exit 2
fi
if [[ ! -f "$config_path" || ! -f "$environment_path" ]]; then
  echo "config and service environment files must exist" >&2
  exit 1
fi

script_dir=$(cd "$(dirname "$0")" && pwd)
exec python3 "$script_dir/merge_provider_config.py" \
  --config "$config_path" \
  --env-file "$environment_path" \
  --proxy-url "$proxy_url"
