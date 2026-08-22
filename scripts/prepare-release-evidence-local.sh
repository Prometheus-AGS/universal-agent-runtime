#!/usr/bin/env bash
set -euo pipefail

# Produce signed release evidence from already-built platform archives and an
# already-published digest-addressed candidate image. This script never builds,
# tags, uploads, or promotes release payloads.

input_directory="${1:?usage: prepare-release-evidence-local.sh <archive-directory> <evidence-directory>}"
evidence_directory="${2:?usage: prepare-release-evidence-local.sh <archive-directory> <evidence-directory>}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

for command in cosign git jq node sha256sum syft tar unzip; do
  command -v "$command" >/dev/null || { echo "missing command: $command" >&2; exit 1; }
done
for name in RELEASE_TAG GA_TAG SUPERSEDED_GA_SHA REPOSITORY IMAGE_REFERENCE IMAGE_DIGEST \
  COSIGN_CERTIFICATE_IDENTITY COSIGN_CERTIFICATE_OIDC_ISSUER TEST_EVIDENCE_FILE \
  SECURITY_AUDIT_EVIDENCE_FILE BUILD_RECEIPT_FILE; do
  [[ -n "${!name:-}" ]] || { echo "missing required environment variable: $name" >&2; exit 1; }
done

[[ "$RELEASE_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+-rc\.[0-9]+$ ]] || {
  echo "RELEASE_TAG must identify an immutable release candidate" >&2
  exit 1
}
[[ "$IMAGE_DIGEST" =~ ^sha256:[0-9a-f]{64}$ ]] || { echo "invalid IMAGE_DIGEST" >&2; exit 1; }
[[ -d "$input_directory" ]] || { echo "archive directory does not exist: $input_directory" >&2; exit 1; }
[[ ! -e "$evidence_directory" ]] || { echo "evidence directory must not already exist: $evidence_directory" >&2; exit 1; }
evidence_parent="$(cd "$(dirname "$evidence_directory")" && pwd)"
evidence_absolute="$evidence_parent/$(basename "$evidence_directory")"
case "$evidence_absolute" in
  "$root/target" | "$root/target/"*) ;;
  "$root" | "$root/"*)
    echo "evidence inside the checkout must be written under target/: $evidence_absolute" >&2
    exit 1
    ;;
esac

source_sha="$(git -C "$root" rev-parse HEAD)"
[[ -z "$(git -C "$root" status --porcelain)" ]] || {
  echo "checkout must be clean before release evidence is produced" >&2
  exit 1
}
[[ -z "$(git -C "$root" submodule status --recursive | awk '$1 ~ /^[+-U]/ { print }')" ]] || {
  echo "recursive submodule pins do not match the source commit" >&2
  exit 1
}

for source in "$TEST_EVIDENCE_FILE" "$SECURITY_AUDIT_EVIDENCE_FILE" "$BUILD_RECEIPT_FILE"; do
  [[ -f "$source" ]] || { echo "required local receipt is missing: $source" >&2; exit 1; }
  [[ "$(jq -er '.source_sha' "$source")" == "$source_sha" ]] || {
    echo "receipt is not bound to source $source_sha: $source" >&2
    exit 1
  }
done

mkdir -p "$evidence_directory"
archive_count=0
while IFS= read -r archive; do
  [[ "$(basename "$archive")" == uar-offline-source.tar.gz ]] && continue
  cp "$archive" "$evidence_directory/"
  archive_count=$((archive_count + 1))
done < <(find "$input_directory" -maxdepth 1 -type f \( -name '*.tar.gz' -o -name '*.zip' \) | sort)
((archive_count > 0)) || { echo "no platform archives found in $input_directory" >&2; exit 1; }

"$root/scripts/package-offline-source.sh" "$evidence_directory/uar-offline-source.tar.gz"
cp "$TEST_EVIDENCE_FILE" "$evidence_directory/test-evidence.json"
cp "$SECURITY_AUDIT_EVIDENCE_FILE" "$evidence_directory/security-audit-evidence.json"
cp "$BUILD_RECEIPT_FILE" "$evidence_directory/build-receipt.json"
cp "$root/scripts/validate-release-manifest.mjs" "$evidence_directory/verify-release.mjs"
cp "$root/schemas/release-manifest.schema.json" "$evidence_directory/release-manifest.schema.json"
cp "$root/docs/product-support-matrix.json" "$evidence_directory/product-support-matrix.json"

scratch="$(mktemp -d "${TMPDIR:-/tmp}/uar-release-evidence.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
for archive in "$evidence_directory"/*.tar.gz "$evidence_directory"/*.zip; do
  [[ -f "$archive" ]] || continue
  name="$(basename "$archive")"
  unpack="$scratch/${name//[^A-Za-z0-9._-]/_}"
  mkdir -p "$unpack"
  case "$archive" in
    *.zip) unzip -q "$archive" -d "$unpack" ;;
    *) tar -xzf "$archive" -C "$unpack" ;;
  esac
  syft "dir:$unpack" -o "cyclonedx-json=${archive}.cyclonedx.json" -o "spdx-json=${archive}.spdx.json"
  if [[ "$name" != uar-offline-source.tar.gz ]]; then
    binary_name=universal-agent-runtime
    [[ "$name" == *.zip ]] && binary_name=universal-agent-runtime.exe
    binary="$(find "$unpack" -type f -name "$binary_name" -print)"
    [[ "$(printf '%s\n' "$binary" | grep -c .)" == 1 ]] || {
      echo "$name must contain exactly one $binary_name" >&2
      exit 1
    }
    syft "file:$binary" -o "cyclonedx-json=${archive}.binary.cyclonedx.json" -o "spdx-json=${archive}.binary.spdx.json"
  fi
done
syft "dir:$root" --exclude target \
  -o "cyclonedx-json=$evidence_directory/source.cyclonedx.json" \
  -o "spdx-json=$evidence_directory/source.spdx.json"
syft "$IMAGE_REFERENCE@$IMAGE_DIGEST" \
  -o "cyclonedx-json=$evidence_directory/image.cyclonedx.json" \
  -o "spdx-json=$evidence_directory/image.spdx.json"

build_started_at="$(jq -er '.started_at' "$evidence_directory/build-receipt.json")"
build_finished_at="$(jq -er '.finished_at' "$evidence_directory/build-receipt.json")"
builder_identity="scripts/prepare-release-evidence-local.sh@$source_sha"
SOURCE_SHA="$source_sha" REPOSITORY="$REPOSITORY" BUILDER_IDENTITY="$builder_identity" \
  RELEASE_TAG="$RELEASE_TAG" IMAGE_REFERENCE="$IMAGE_REFERENCE" IMAGE_DIGEST="$IMAGE_DIGEST" \
  BUILD_STARTED_AT="$build_started_at" BUILD_FINISHED_AT="$build_finished_at" \
  node "$root/scripts/generate-local-image-provenance.mjs" "$evidence_directory/image.provenance.json"
for archive in "$evidence_directory"/*.tar.gz "$evidence_directory"/*.zip; do
  [[ -f "$archive" ]] || continue
  name="$(basename "$archive")"
  SOURCE_SHA="$source_sha" REPOSITORY="$REPOSITORY" BUILDER_IDENTITY="$builder_identity" \
    RELEASE_TAG="$RELEASE_TAG" BUILD_STARTED_AT="$build_started_at" BUILD_FINISHED_AT="$build_finished_at" \
    node "$root/scripts/generate-local-provenance.mjs" "$archive" "$evidence_directory/$name.intoto.jsonl"
  cosign sign-blob --yes --bundle "$evidence_directory/$name.sigstore.json" "$archive"
done
cosign sign --yes "$IMAGE_REFERENCE@$IMAGE_DIGEST"
cosign attest --yes --type slsaprovenance \
  --predicate "$evidence_directory/image.provenance.json" \
  "$IMAGE_REFERENCE@$IMAGE_DIGEST"
cosign attest --yes --type cyclonedx \
  --predicate "$evidence_directory/image.cyclonedx.json" \
  "$IMAGE_REFERENCE@$IMAGE_DIGEST"

SOURCE_SHA="$source_sha" BUILDER_IDENTITY="$builder_identity" BUILD_RECEIPT=build-receipt.json \
  TEST_EVIDENCE=test-evidence.json SECURITY_AUDIT_EVIDENCE=security-audit-evidence.json \
  node "$root/scripts/generate-release-manifest.mjs" "$evidence_directory" "$evidence_directory/release-manifest.json"
(
  cd "$evidence_directory"
  checksum_index="$(mktemp "${TMPDIR:-/tmp}/uar-release-checksums.XXXXXX")"
  find . -maxdepth 1 -type f ! -name SHA256SUMS -print \
    | sed 's#^\./##' \
    | LC_ALL=C sort \
    | while IFS= read -r file; do sha256sum "$file"; done >"$checksum_index"
  mv "$checksum_index" SHA256SUMS
)
cosign sign-blob --yes \
  --bundle "$evidence_directory/SHA256SUMS.sigstore.json" \
  "$evidence_directory/SHA256SUMS"
node "$evidence_directory/verify-release.mjs" "$evidence_directory/release-manifest.json"

echo "Local release evidence prepared for $RELEASE_TAG at $source_sha ($archive_count platform archives)."
echo "No tag, release, archive, or image was built, uploaded, or promoted by this script."
