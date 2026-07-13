#!/usr/bin/env bash
set -euo pipefail

readonly max_attempts="${SUBMODULE_UPDATE_ATTEMPTS:-3}"

# actions/checkout removes its temporary global SSH rewrite after initializing
# top-level submodules. Propagate an equivalent rewrite, plus its persisted
# masked HTTPS authorization header, to every nested Git process.
export GIT_CONFIG_KEY_0='url.https://github.com/.insteadOf'
export GIT_CONFIG_VALUE_0='git@github.com:'
export GIT_CONFIG_KEY_1='url.https://github.com/.insteadOf'
export GIT_CONFIG_VALUE_1='org-208548015@github.com:'
git_config_count=2

auth_header="$(git config --local --get http.https://github.com/.extraheader || true)"
if [[ -n "$auth_header" ]]; then
  export GIT_CONFIG_KEY_2='http.https://github.com/.extraheader'
  export GIT_CONFIG_VALUE_2="$auth_header"
  git_config_count=3
fi
export GIT_CONFIG_COUNT="$git_config_count"

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
