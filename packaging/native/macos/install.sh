#!/usr/bin/env bash
set -euo pipefail

service_label="com.prometheus.universal-agent-runtime"
binary_source=""
static_source=""
config_source=""

usage() {
  echo "usage: $0 --binary <release-binary> --static-dir <react-bundle> [--config <initial-config>]" >&2
}

while (($#)); do
  case "$1" in
    --binary)
      binary_source=${2-}
      shift 2
      ;;
    --static-dir)
      static_source=${2-}
      shift 2
      ;;
    --config)
      config_source=${2-}
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

if [[ ! -f "$binary_source" || ! -d "$static_source" ]]; then
  usage
  exit 2
fi

uar_home="$HOME/.uar"
config_path="$uar_home/config.yaml"
environment_path="$uar_home/service.env"
log_dir="$HOME/.prometheus/logs/universal-agent-runtime"
backup_dir="$HOME/.prometheus/backups/uar"
launch_agents_dir="$HOME/Library/LaunchAgents"
plist_path="$launch_agents_dir/$service_label.plist"
template_path="$(cd "$(dirname "$0")" && pwd)/$service_label.plist.in"

if [[ ! -f "$config_path" && ! -f "$config_source" ]]; then
  echo "first install requires --config <initial-config>" >&2
  exit 2
fi

mkdir -p "$uar_home/bin" "$log_dir" "$backup_dir" "$launch_agents_dir"
chmod 700 "$uar_home" "$uar_home/bin" "$log_dir" "$backup_dir"

timestamp=$(date -u +%Y%m%dT%H%M%SZ)
if [[ -f "$config_path" ]]; then
  cp -p "$config_path" "$backup_dir/config.yaml.$timestamp"
elif [[ -n "$config_source" ]]; then
  install -m 600 "$config_source" "$config_path"
fi

binary_tmp="$uar_home/bin/.universal-agent-runtime.$$.tmp"
install -m 755 "$binary_source" "$binary_tmp"
mv -f "$binary_tmp" "$uar_home/bin/universal-agent-runtime"

static_tmp="$uar_home/.static.$$.tmp"
rm -rf "$static_tmp"
mkdir -p "$static_tmp"
cp -R "$static_source"/. "$static_tmp"/
if [[ -d "$uar_home/static" ]]; then
  mv "$uar_home/static" "$backup_dir/static.$timestamp"
fi
mv "$static_tmp" "$uar_home/static"

if [[ ! -f "$environment_path" ]]; then
  umask 077
  {
    echo "UAR_SERVER__HOST=127.0.0.1"
    echo "UAR_SERVER__GRPC_PORT=50051"
    echo "PORT=1906"
    echo "UAR_LOG_FILE=$log_dir/operational.log"
  } > "$environment_path"
fi
chmod 600 "$environment_path" "$config_path"

escape_sed() {
  printf '%s' "$1" | sed 's/[&|\\]/\\&/g'
}

escaped_home=$(escape_sed "$uar_home")
escaped_log_dir=$(escape_sed "$log_dir")
plist_tmp="$plist_path.$$.tmp"
sed \
  -e "s|__UAR_HOME__|$escaped_home|g" \
  -e "s|__LOG_DIR__|$escaped_log_dir|g" \
  "$template_path" > "$plist_tmp"
chmod 600 "$plist_tmp"
mv -f "$plist_tmp" "$plist_path"

launchctl bootout "gui/$UID" "$plist_path" >/dev/null 2>&1 || true
launchctl bootstrap "gui/$UID" "$plist_path"
launchctl kickstart -k "gui/$UID/$service_label"
echo "installed $service_label"
