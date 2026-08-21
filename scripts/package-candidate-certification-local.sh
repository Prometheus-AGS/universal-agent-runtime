#!/usr/bin/env bash
set -euo pipefail

# Package already-observed local candidate-certification results. This script
# signs the checksum root but never uploads assets or mutates a release.

results_directory="${1:?usage: package-candidate-certification-local.sh <results-directory> <supply-evidence-directory> <output-directory>}"
supply_directory="${2:?usage: package-candidate-certification-local.sh <results-directory> <supply-evidence-directory> <output-directory>}"
output_directory="${3:?usage: package-candidate-certification-local.sh <results-directory> <supply-evidence-directory> <output-directory>}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

for command in cosign git jq node sha256sum tar; do
  command -v "$command" >/dev/null || { echo "missing command: $command" >&2; exit 1; }
done
[[ -d "$results_directory" ]] || { echo "results directory does not exist: $results_directory" >&2; exit 1; }
[[ -d "$supply_directory" ]] || { echo "supply evidence directory does not exist: $supply_directory" >&2; exit 1; }
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

source_sha="$(git -C "$root" rev-parse HEAD)"
[[ -z "$(git -C "$root" status --porcelain)" ]] || {
  echo "checkout must be clean before candidate certification is packaged" >&2
  exit 1
}
manifest="$supply_directory/release-manifest.json"
index="$supply_directory/SHA256SUMS"
[[ -f "$manifest" && -f "$index" ]] || { echo "supply evidence is incomplete" >&2; exit 1; }
node "$root/scripts/validate-release-manifest.mjs" "$manifest"
node "$root/scripts/validate-candidate-certification.mjs" "$results_directory"

candidate="$(jq -er '.release' "$manifest")"
manifest_source="$(jq -er '.source.sha' "$manifest")"
result_source="$(jq -er '.source_sha' "$results_directory/results.json")"
result_candidate="$(jq -er '.candidate_tag' "$results_directory/results.json")"
[[ "$manifest_source" == "$source_sha" && "$result_source" == "$source_sha" ]] || {
  echo "candidate certification is not bound to checkout source $source_sha" >&2
  exit 1
}
[[ "$result_candidate" == "$candidate" ]] || { echo "candidate certification tag mismatch" >&2; exit 1; }

mkdir -p "$output_directory"
archive="candidate-certification-${candidate}.tar.gz"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/uar-candidate-certification.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/release-candidate-certification"
cp -R "$results_directory/." "$scratch/release-candidate-certification/"
tar -czf "$output_directory/$archive" -C "$scratch" release-candidate-certification
archive_sha="$(sha256sum "$output_directory/$archive" | cut -d' ' -f1)"
release_manifest_sha="$(sha256sum "$manifest" | cut -d' ' -f1)"
supply_index_sha="$(sha256sum "$index" | cut -d' ' -f1)"
receipt_sha="$(sha256sum "$results_directory/results.json" | cut -d' ' -f1)"
certificate_identity="$(jq -er '.signing.certificate_identity' "$manifest")"
certificate_issuer="$(jq -er '.signing.certificate_oidc_issuer' "$manifest")"
builder_identity="scripts/package-candidate-certification-local.sh@$source_sha"

jq -n \
  --arg candidate "$candidate" \
  --arg source_sha "$source_sha" \
  --arg builder_identity "$builder_identity" \
  --arg receipt "results.json" \
  --arg receipt_sha "$receipt_sha" \
  --arg certificate_identity "$certificate_identity" \
  --arg certificate_issuer "$certificate_issuer" \
  --arg release_manifest_sha "$release_manifest_sha" \
  --arg supply_index_sha "$supply_index_sha" \
  --arg archive "$archive" \
  --arg archive_sha "$archive_sha" \
  '{schema_version:1,candidate:$candidate,source_sha:$source_sha,builder:{kind:"local",identity:$builder_identity,source_sha:$source_sha,receipt:$receipt,receipt_sha256:$receipt_sha},signing:{certificate_identity:$certificate_identity,certificate_oidc_issuer:$certificate_issuer},supply:{release_manifest_sha256:$release_manifest_sha,checksum_index_sha256:$supply_index_sha},archive:{name:$archive,sha256:$archive_sha}}' \
  >"$output_directory/candidate-certification-manifest.json"
(
  cd "$output_directory"
  sha256sum "$archive" candidate-certification-manifest.json >CANDIDATE_CERTIFICATION_SHA256SUMS
)
cosign sign-blob --yes \
  --bundle "$output_directory/CANDIDATE_CERTIFICATION_SHA256SUMS.sigstore.json" \
  "$output_directory/CANDIDATE_CERTIFICATION_SHA256SUMS"
node "$root/scripts/validate-candidate-certification-bundle.mjs" "$output_directory" "$manifest"

echo "Local candidate certification package prepared for $candidate at $source_sha."
echo "No candidate tag, release, archive, or image was built, uploaded, or promoted by this script."
