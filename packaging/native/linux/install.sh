#!/usr/bin/env bash
set -euo pipefail

binary_source=""
static_source=""
config_source=""
script_dir=$(cd "$(dirname "$0")" && pwd)

usage() {
  echo "usage: sudo $0 --binary <release-binary> --static-dir <react-bundle> [--config <initial-config>]" >&2
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

if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
  echo "install must run as root" >&2
  exit 1
fi
if [[ ! -f "$binary_source" || ! -d "$static_source" ]]; then
  usage
  exit 2
fi

config_dir="/etc/uar"
config_path="$config_dir/config.yaml"
environment_path="$config_dir/uar.env"
state_dir="/var/lib/uar"
log_dir="$state_dir/.prometheus/logs"
backup_dir="$state_dir/.prometheus/backups"
program_dir="/usr/local/lib/uar"
unit_path="/etc/systemd/system/uar.service"
default_config="$script_dir/../default-config.yaml"
environment_generator="$script_dir/../common/generate-provider-env.sh"
config_merger="$script_dir/../common/merge-provider-config.sh"

if [[ ! -f "$config_path" && -z "$config_source" ]]; then
  config_source="$default_config"
fi
if [[ ! -f "$config_path" && ! -f "$config_source" ]]; then
  echo "initial configuration not found" >&2
  exit 1
fi

if ! getent group uar >/dev/null; then
  groupadd --system uar
fi
if ! id uar >/dev/null 2>&1; then
  useradd --system --gid uar --home-dir "$state_dir" --shell /usr/sbin/nologin uar
fi

install -d -m 750 -o root -g uar "$config_dir"
install -d -m 755 -o root -g root "$program_dir"
install -d -m 750 -o uar -g uar "$state_dir" "$state_dir/.prometheus" "$log_dir" "$backup_dir"

timestamp=$(date -u +%Y%m%dT%H%M%SZ)
if [[ -f "$config_path" ]]; then
  install -m 600 -o uar -g uar "$config_path" "$backup_dir/config.yaml.$timestamp"
elif [[ -n "$config_source" ]]; then
  install -m 640 -o root -g uar "$config_source" "$config_path"
fi

systemctl stop uar.service >/dev/null 2>&1 || true
install -m 755 -o root -g root "$binary_source" "$program_dir/universal-agent-runtime"

static_tmp="$program_dir/.static.$$.tmp"
rm -rf "$static_tmp"
install -d -m 755 -o root -g root "$static_tmp"
cp -R "$static_source"/. "$static_tmp"/
rm -rf "$program_dir/static"
mv "$static_tmp" "$program_dir/static"
chown -R root:root "$program_dir/static"

if [[ ! -f "$environment_path" ]]; then
  umask 027
  {
    echo "UAR_SERVER__HOST=127.0.0.1"
    echo "UAR_SERVER__GRPC_PORT=50051"
    echo "PORT=1906"
    echo "UAR_LOG_FILE=$log_dir/operational.log"
  } > "$environment_path"
fi
"$environment_generator" --output "$environment_path"
"$config_merger" \
  --config "$config_path" \
  --env-file "$environment_path" \
  --proxy-url "http://127.0.0.1:8181/v1"
chown root:uar "$environment_path" "$config_path"
chmod 640 "$environment_path" "$config_path"

install -m 644 -o root -g root "$script_dir/uar.service" "$unit_path"
systemctl daemon-reload
systemctl enable --now uar.service
echo "installed uar.service"
