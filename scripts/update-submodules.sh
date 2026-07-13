#!/usr/bin/env bash
set -euo pipefail

readonly max_attempts="${SUBMODULE_UPDATE_ATTEMPTS:-3}"

git submodule sync --recursive

for attempt in $(seq 1 "$max_attempts"); do
  if git -c protocol.version=2 submodule update --init --force --recursive; then
    git submodule status --recursive
    exit 0
  fi

  if [[ "$attempt" -eq "$max_attempts" ]]; then
    echo "Recursive submodule checkout failed after ${max_attempts} attempts." >&2
    exit 1
  fi

  echo "Recursive submodule checkout attempt ${attempt}/${max_attempts} failed; retrying." >&2
  git submodule sync --recursive
  sleep "$((attempt * 2))"
done
