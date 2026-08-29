#!/usr/bin/env bash
set -euo pipefail

# Run release-blocking security checks locally and retain a source-bound receipt.
# GitHub Actions are reserved for deployment execution and validation.

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

check_documentation_image_inputs() {
  local -a documentation_inputs=()
  local candidate
  local affected_input
  local mime_type

  for candidate in \
    "$root/docs/adr" \
    "$root/website/docs" \
    "$root/website/src" \
    "$root/website/static"; do
    [[ -d "$candidate" ]] && documentation_inputs+=("$candidate")
  done

  affected_input="$(find "${documentation_inputs[@]}" -type f \
    \( -iname '*.icns' -o -iname '*.jxl' -o -iname '*.heif' -o \
       -iname '*.heic' -o -iname '*.avif' \) -print -quit)"
  if [[ -n "$affected_input" ]]; then
    echo "unsupported documentation image input: $affected_input" >&2
    echo "ICNS, JXL, HEIF, HEIC, and AVIF are blocked while image-size has no patched release" >&2
    return 1
  fi

  command -v file >/dev/null || {
    echo "missing command required for documentation image content inspection: file" >&2
    return 1
  }
  while IFS= read -r -d '' candidate; do
    if ! mime_type="$(file --brief --mime-type -- "$candidate")"; then
      echo "could not inspect documentation image input: $candidate" >&2
      return 1
    fi
    case "$mime_type" in
      image/icns | image/x-icns | image/jxl | image/heif | image/heic | image/avif)
        echo "unsupported documentation image content ($mime_type): $candidate" >&2
        echo "ICNS, JXL, HEIF, HEIC, and AVIF are blocked while image-size has no patched release" >&2
        return 1
        ;;
    esac
  done < <(find "${documentation_inputs[@]}" -type f -print0)
}

check_rkyv_advisory_inactive() {
  local metadata
  local inverse_tree

  for command in cargo jq; do
    command -v "$command" >/dev/null || {
      echo "missing command required for rkyv advisory inspection: $command" >&2
      return 1
    }
  done

  if ! metadata="$(cargo metadata --manifest-path "$root/Cargo.toml" --locked --format-version 1)"; then
    echo "could not resolve the locked Cargo graph for the rkyv advisory check" >&2
    return 1
  fi
  if ! jq -e '.packages[] | select(.name == "rkyv" and .version == "0.7.46")' \
    <<<"$metadata" >/dev/null; then
    echo "rkyv 0.7.46 is absent from locked package metadata; no advisory exception is active."
    return 0
  fi

  if ! inverse_tree="$(cargo tree --manifest-path "$root/Cargo.toml" \
    --locked --all-features --target all --edges all -i rkyv@0.7.46)"; then
    echo "could not inspect reverse dependencies for rkyv 0.7.46" >&2
    return 1
  fi
  if [[ -n "$inverse_tree" ]]; then
    echo "$inverse_tree" >&2
    echo "RUSTSEC-2026-0235 may be ignored only while rkyv 0.7.46 is inactive for every supported target and feature" >&2
    return 1
  fi

  echo "rkyv 0.7.46 is lockfile-only and inactive for all targets and feature edges."
}

audit_website_dependencies() {
  local audit_json
  local audit_status=0

  audit_json="$(mktemp "${TMPDIR:-/tmp}/uar-website-audit.XXXXXX")"
  npm --prefix "$root/website" audit --json >"$audit_json" 2>&1 || audit_status=$?

  if jq -e '
    ([.vulnerabilities[]?.via[]? | objects | .source] | sort | unique) as $sources
    | (($sources == []) and (.metadata.vulnerabilities.total == 0))
      or (($sources == [1138808, 1138809])
          and (.vulnerabilities["image-size"].fixAvailable == false))
  ' "$audit_json" >/dev/null; then
    cat "$audit_json"
    if [[ "$audit_status" -ne 0 ]]; then
      echo "Only the two approved, unpatched image-size build-input advisories remain."
    fi
    rm -f "$audit_json"
    return 0
  fi

  cat "$audit_json" >&2
  rm -f "$audit_json"
  echo "website dependency audit contains an advisory outside the bounded image-size exception" >&2
  return 1
}

if [[ "${1:-}" == "--check-doc-image-inputs-only" ]]; then
  check_documentation_image_inputs
  echo "Documentation image inputs passed the image-size advisory gate."
  exit 0
fi

if [[ "${1:-}" == "--check-website-audit-only" ]]; then
  audit_website_dependencies
  echo "Website dependencies contain no unaccepted advisories."
  exit 0
fi

if [[ "${1:-}" == "--check-rkyv-advisory-only" ]]; then
  check_rkyv_advisory_inactive
  echo "The rkyv advisory exception is mechanically bounded to an inactive lockfile entry."
  exit 0
fi

output_directory="${1:?usage: security-audit-local.sh <output-directory> | --check-doc-image-inputs-only | --check-website-audit-only | --check-rkyv-advisory-only}"
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

run_check rkyv-0.7-inactive check_rkyv_advisory_inactive
run_check cargo-audit cargo audit \
  --ignore RUSTSEC-2026-0194 \
  --ignore RUSTSEC-2026-0195 \
  --ignore RUSTSEC-2026-0235 \
  --ignore RUSTSEC-2023-0071
run_check pnpm-root-audit pnpm audit
run_check pnpm-frontend-audit pnpm -C frontend audit
run_check npm-website-audit audit_website_dependencies
run_check npm-typescript-sdk-audit npm --prefix sdks/typescript audit
run_check documentation-image-inputs check_documentation_image_inputs
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
