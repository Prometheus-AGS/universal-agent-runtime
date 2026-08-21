#!/usr/bin/env bash
set -euo pipefail

event_name="${1:?GitHub event name is required}"
dispatch_duration="${2:-}"

case "$event_name" in
  pull_request)
    duration=60
    ;;
  workflow_dispatch)
    duration="$dispatch_duration"
    ;;
  schedule)
    duration=10800
    ;;
  *)
    echo "unsupported operational-resilience event: $event_name" >&2
    exit 1
    ;;
esac

[[ "$duration" =~ ^[0-9]+$ ]] || {
  echo "soak duration must be a non-negative integer" >&2
  exit 1
}

if [[ "$event_name" != "pull_request" ]] && (( duration < 10800 )); then
  echo "certifying soak duration must be at least 10800 seconds" >&2
  exit 1
fi

printf '%s\n' "$duration"
