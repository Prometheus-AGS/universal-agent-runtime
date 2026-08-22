#!/usr/bin/env bash
set -euo pipefail

# Run release-blocking security checks locally and retain a source-bound receipt.
# GitHub Actions are reserved for deployment execution and validation.

output_directory="${1:?usage: security-audit-local.sh <output-directory>}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
repository="${GITHUB_REPOSITORY:-Prometheus-AGS/universal-agent-runtime}"
image="${UAR_SECURITY_IMAGE:?UAR_SECURITY_IMAGE must be a digest-addressed candidate image}"

for command in cargo cargo-audit gh grype jq npm osv-scanner pnpm sha256sum; do
  command -v "$command" >/dev/null || { echo "missing command: $command" >&2; exit 1; }
done
[[ "$image" =~ @sha256:[0-9a-f]{64}$ ]] || { echo "UAR_SECURITY_IMAGE must be digest-addressed" >&2; exit 1; }
[[ ! -e "$output_directory" ]] || { echo "output directory must not already exist: $output_directory" >&2; exit 1; }
output_parent="$(cd "$(dirname "$output_directory")" && pwd)"
output_absolute="$output_parent/$(basename "$output_directory")"
case "$output_absolute" in
  "$root/target" | "$root/target/"*) ;;
  "$root" | "$root/"*)
    echo "output inside the checkout must be written under target/: $output_absolute" >&2
    exit 1
    ;;
esac
[[ -n "${GH_TOKEN:-}" ]] || { echo "GH_TOKEN with Dependabot alerts read access is required" >&2; exit 1; }
[[ -z "$(git -C "$root" status --porcelain)" ]] || { echo "checkout must be clean before security certification" >&2; exit 1; }

source_sha="$(git -C "$root" rev-parse HEAD)"
started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
mkdir -p "$output_directory"

run_check() {
  local name="$1"
  shift
  echo "Running local security check: $name"
  if ! "$@" >"$output_directory/$name.log" 2>&1; then
    cat "$output_directory/$name.log" >&2
    echo "local security check failed: $name" >&2
    exit 1
  fi
}

run_check cargo-audit cargo audit \
  --ignore RUSTSEC-2026-0194 \
  --ignore RUSTSEC-2026-0195 \
  --ignore RUSTSEC-2023-0071
run_check pnpm-root-audit pnpm audit
run_check pnpm-frontend-audit pnpm -C frontend audit
run_check npm-typescript-sdk-audit npm --prefix sdks/typescript audit
run_check osv-source-scan osv-scanner --recursive --skip-git "$root"
run_check grype-image-scan grype "$image" --fail-on high

gh api "repos/$repository/dependabot/alerts" --paginate --slurp \
  >"$output_directory/dependabot-alerts.raw.json"
jq '[.[][] | select(.state == "open")]' "$output_directory/dependabot-alerts.raw.json" \
  >"$output_directory/dependabot-alerts.open.json"
open_alerts="$(jq 'length' "$output_directory/dependabot-alerts.open.json")"
[[ "$open_alerts" == 0 ]] || {
  jq -r '.[] | "\(.security_advisory.ghsa_id) (\(.security_advisory.severity)) -- \(.dependency.package.name)"' \
    "$output_directory/dependabot-alerts.open.json" >&2
  echo "$open_alerts open Dependabot alert(s) require triage; no inline allowlist exists" >&2
  exit 1
}

finished_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq -n \
  --arg source_sha "$source_sha" \
  --arg image "$image" \
  --arg started_at "$started_at" \
  --arg finished_at "$finished_at" \
  --argjson open_dependabot_alerts "$open_alerts" \
  '{schema_version:1,source_sha:$source_sha,image:$image,started_at:$started_at,finished_at:$finished_at,outcome:"passed",open_dependabot_alerts:$open_dependabot_alerts}' \
  >"$output_directory/security-audit-evidence.json"
(
  cd "$output_directory"
  checksum_index="$(mktemp "${TMPDIR:-/tmp}/uar-security-checksums.XXXXXX")"
  find . -maxdepth 1 -type f ! -name SHA256SUMS -print \
    | sed 's#^\./##' \
    | LC_ALL=C sort \
    | while IFS= read -r file; do sha256sum "$file"; done >"$checksum_index"
  mv "$checksum_index" SHA256SUMS
)

echo "Local security audit passed for $source_sha and $image."
