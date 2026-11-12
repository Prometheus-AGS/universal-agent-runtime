#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output="${1:-$root/dist/uar-offline-source.tar.gz}"
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT

mkdir -p "$stage/source" "$(dirname "$output")"
(
  cd "$root"
  git ls-files --recurse-submodules -z \
    | while IFS= read -r -d '' source_path; do
        [[ "$source_path" == ".claude/settings.local.json" \
          || "$source_path" == */.claude/settings.local.json ]] && continue
        printf '%s\0' "$source_path"
      done \
    | tar --null -T - -cf -
) | tar -xf - -C "$stage/source"

mkdir -p "$stage/source/.cargo"
(
  cd "$stage/source"
  cargo vendor --locked vendor/crates >../vendor-config.toml
  if [[ ! -s ../vendor-config.toml ]]; then
    printf 'cargo vendor did not emit source replacement configuration\n' >&2
    exit 1
  fi
  mv ../vendor-config.toml .cargo/config.toml
)

tar -czf "$output" -C "$stage/source" .
printf 'Created %s\n' "$output"
shasum -a 256 "$output"
