#!/usr/bin/env bash
set -euo pipefail

# Produce signed release evidence from already-built platform archives and an
# already-published digest-addressed candidate image. This script never builds,
# tags, uploads, or promotes release payloads.

input_directory="${1:?usage: prepare-release-evidence-local.sh <archive-directory> <evidence-directory>}"
evidence_directory="${2:?usage: prepare-release-evidence-local.sh <archive-directory> <evidence-directory>}"
for command in git jq; do
  command -v "$command" >/dev/null || { echo "missing command: $command" >&2; exit 1; }
done
tools="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "${UAR_RELEASE_SOURCE_ROOT:-$tools/..}" && pwd)"
[[ "$(git -C "$root" rev-parse --show-toplevel)" == "$root" ]] || {
  echo "release source root must be the Git checkout root" >&2
  exit 1
}

for name in RELEASE_TAG GA_TAG SUPERSEDED_GA_SHA REPOSITORY IMAGE_REFERENCE IMAGE_DIGEST \
  COSIGN_CERTIFICATE_IDENTITY COSIGN_CERTIFICATE_OIDC_ISSUER TEST_EVIDENCE_FILE \
  SECURITY_AUDIT_EVIDENCE_FILE BUILD_RECEIPT_FILE; do
  [[ -n "${!name:-}" ]] || { echo "missing required environment variable: $name" >&2; exit 1; }
done

[[ "$RELEASE_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+-rc\.[0-9]+$ ]] || {
  echo "RELEASE_TAG must identify an immutable release candidate" >&2
  exit 1
}
[[ "$GA_TAG" == "${RELEASE_TAG%-rc.*}" && ( "$SUPERSEDED_GA_SHA" == null || "$SUPERSEDED_GA_SHA" =~ ^[0-9a-f]{40}$ ) ]] || {
  echo "GA tag or superseded commit does not match the release promotion contract" >&2
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
source_tree="$(git -C "$root" rev-parse 'HEAD^{tree}')"
image="$IMAGE_REFERENCE@$IMAGE_DIGEST"
source_status="$(git -C "$root" status --porcelain --untracked-files=all --ignore-submodules=none)"
[[ -z "$source_status" ]] || {
  echo "checkout must be clean before release evidence is produced" >&2
  exit 1
}
submodule_status="$(git -C "$root" submodule status --recursive)"
[[ -z "$(awk 'NF && substr($0, 1, 1) != " " { print }' <<< "$submodule_status")" ]] || {
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

require_passed_audit() {
  jq -e --arg source "$source_sha" --arg image "$image" --arg repository "$REPOSITORY" '
    .source_sha == $source and .repository == $repository and .outcome == "passed" and
    .image == $image and .open_dependabot_alerts == 0
  ' "$1" >/dev/null || {
    echo "security audit must have passed for the exact release image with zero open Dependabot alerts" >&2
    exit 1
  }
}
require_passed_audit "$SECURITY_AUDIT_EVIDENCE_FILE"

require_artifact_receipt() {
  jq -e --arg source "$source_sha" --arg tree "$source_tree" --arg image "$image" '
    .source_sha == $source and .source_tree == $tree and .image == $image
  ' "$1" >/dev/null || {
    echo "receipt is not bound to release source tree $source_tree and image $image: $1" >&2
    exit 1
  }
}
require_artifact_receipt "$BUILD_RECEIPT_FILE"
require_artifact_receipt "$TEST_EVIDENCE_FILE"

for command in cosign docker node sha256sum syft tar unzip; do
  command -v "$command" >/dev/null || { echo "missing command: $command" >&2; exit 1; }
done

scratch="$(mktemp -d "${TMPDIR:-/tmp}/uar-release-evidence.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
image_index="$scratch/image.index.json"
docker buildx imagetools inspect --raw "$image" > "$image_index"
platforms="$(node --input-type=module -e '
  import { readFileSync } from "node:fs";
  import { pathToFileURL } from "node:url";
  const [modulePath, index, digest] = process.argv.slice(1);
  const { releaseImagePlatforms } = await import(pathToFileURL(modulePath));
  console.log(releaseImagePlatforms(readFileSync(index), digest).join("\n"));
' "$tools/release-image-index.mjs" "$image_index" "$IMAGE_DIGEST")"

if [[ "$RELEASE_TAG" == v1.4.0-rc.4 && "$platforms" != linux/amd64 ]]; then
  echo "RC4 requires exactly the selected Linux amd64 image platform" >&2
  exit 1
fi

if [[ "$platforms" == linux/amd64 ]]; then
  bound_archive="$(jq -er '.archive | strings' "$BUILD_RECEIPT_FILE")"
  archive_sha="$(jq -er '.archive_sha256 | strings' "$BUILD_RECEIPT_FILE")"
  [[ "$bound_archive" =~ ^[a-zA-Z0-9_-][a-zA-Z0-9_.-]*\.tar\.gz$ &&
     "$bound_archive" != uar-offline-source.tar.gz && "$archive_sha" =~ ^[0-9a-f]{64}$ &&
     -f "$input_directory/$bound_archive" && ! -L "$input_directory/$bound_archive" ]] || {
    echo "Linux amd64 build receipt must bind its exact platform archive" >&2
    exit 1
  }
  actual_archive_sha="$(sha256sum -- "$input_directory/$bound_archive" | cut -d ' ' -f1)"
  [[ "$actual_archive_sha" == "$archive_sha" &&
     "$(jq -er '.archive_sha256 | strings' "$TEST_EVIDENCE_FILE")" == "$archive_sha" ]] || {
    echo "Linux amd64 archive differs from build or test receipt" >&2
    exit 1
  }
fi

mkdir -p "$evidence_directory"
cp "$image_index" "$evidence_directory/image.index.json"
node "$root/scripts/generate-release-input-provenance.mjs" "$evidence_directory/release-input-provenance.json" "$root"
archive_count=0
while IFS= read -r archive; do
  archive_name="$(basename "$archive")"
  [[ "$archive_name" == uar-offline-source.tar.gz ]] && continue
  if [[ "$platforms" == linux/amd64 && "$archive_name" != "$bound_archive" ]]; then
    echo "unexpected archive for selected Linux amd64 release: $archive_name" >&2
    exit 1
  fi
  cp "$archive" "$evidence_directory/"
  archive_count=$((archive_count + 1))
done < <(find "$input_directory" -maxdepth 1 -type f \( -name '*.tar.gz' -o -name '*.zip' \) | sort)
((archive_count > 0)) || { echo "no platform archives found in $input_directory" >&2; exit 1; }

"$root/scripts/package-offline-source.sh" "$evidence_directory/uar-offline-source.tar.gz"
cp "$TEST_EVIDENCE_FILE" "$evidence_directory/test-evidence.json"
cp "$SECURITY_AUDIT_EVIDENCE_FILE" "$evidence_directory/security-audit-evidence.json"
require_passed_audit "$evidence_directory/security-audit-evidence.json"
cp "$BUILD_RECEIPT_FILE" "$evidence_directory/build-receipt.json"
require_artifact_receipt "$evidence_directory/build-receipt.json"
require_artifact_receipt "$evidence_directory/test-evidence.json"
if [[ "$platforms" == linux/amd64 ]]; then
  [[ "$(sha256sum -- "$evidence_directory/$bound_archive" | cut -d ' ' -f1)" == "$archive_sha" ]] &&
    jq -e --arg archive "$bound_archive" --arg sha "$archive_sha" \
      '.archive == $archive and .archive_sha256 == $sha' "$evidence_directory/build-receipt.json" >/dev/null &&
    jq -e --arg sha "$archive_sha" \
      '.archive_sha256 == $sha' "$evidence_directory/test-evidence.json" >/dev/null || {
    echo "copied Linux amd64 archive or receipt changed before signing" >&2
    exit 1
  }
fi
cp "$tools/validate-release-manifest.mjs" "$evidence_directory/verify-release.mjs"
cp "$tools/release-image-index.mjs" "$evidence_directory/release-image-index.mjs"
cp "$tools/../schemas/release-manifest.schema.json" "$evidence_directory/release-manifest.schema.json"
cp "$root/docs/product-support-matrix.json" "$evidence_directory/product-support-matrix.json"

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
tool_digest="$(cat "$tools"/{prepare-release-evidence-local.sh,generate-release-manifest.mjs,release-image-index.mjs,validate-release-manifest.mjs} | sha256sum | cut -d ' ' -f1)"
builder_identity="scripts/prepare-release-evidence-local.sh@sha256:$tool_digest"
SOURCE_SHA="$source_sha" REPOSITORY="$REPOSITORY" BUILDER_IDENTITY="$builder_identity" \
  RELEASE_TAG="$RELEASE_TAG" IMAGE_REFERENCE="$IMAGE_REFERENCE" IMAGE_DIGEST="$IMAGE_DIGEST" \
  BUILD_STARTED_AT="$build_started_at" BUILD_FINISHED_AT="$build_finished_at" \
  node "$root/scripts/generate-local-image-provenance.mjs" "$evidence_directory/image.provenance.json"
verify_signed_blob() {
  cosign verify-blob --bundle "$2" \
    --certificate-identity "$COSIGN_CERTIFICATE_IDENTITY" \
    --certificate-oidc-issuer "$COSIGN_CERTIFICATE_OIDC_ISSUER" "$1"
}
for archive in "$evidence_directory"/*.tar.gz "$evidence_directory"/*.zip; do
  [[ -f "$archive" ]] || continue
  name="$(basename "$archive")"
  SOURCE_SHA="$source_sha" REPOSITORY="$REPOSITORY" BUILDER_IDENTITY="$builder_identity" \
    RELEASE_TAG="$RELEASE_TAG" BUILD_STARTED_AT="$build_started_at" BUILD_FINISHED_AT="$build_finished_at" \
    node "$root/scripts/generate-local-provenance.mjs" "$archive" "$evidence_directory/$name.intoto.jsonl"
  cosign sign-blob --yes --bundle "$evidence_directory/$name.sigstore.json" "$archive"
  verify_signed_blob "$archive" "$evidence_directory/$name.sigstore.json"
done

SOURCE_SHA="$source_sha" BUILDER_IDENTITY="$builder_identity" BUILD_RECEIPT=build-receipt.json \
  TEST_EVIDENCE=test-evidence.json SECURITY_AUDIT_EVIDENCE=security-audit-evidence.json \
  node "$tools/generate-release-manifest.mjs" "$evidence_directory" "$evidence_directory/release-manifest.json" "$root"
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
verify_signed_blob "$evidence_directory/SHA256SUMS" "$evidence_directory/SHA256SUMS.sigstore.json"
node "$evidence_directory/verify-release.mjs" "$evidence_directory/release-manifest.json"
cosign sign --yes "$image"
cosign verify --certificate-identity "$COSIGN_CERTIFICATE_IDENTITY" \
  --certificate-oidc-issuer "$COSIGN_CERTIFICATE_OIDC_ISSUER" "$image"
verify_image_attestation() {
  local type="$1" predicate="$2"
  cosign verify-attestation --type "$type" \
    --certificate-identity "$COSIGN_CERTIFICATE_IDENTITY" \
    --certificate-oidc-issuer "$COSIGN_CERTIFICATE_OIDC_ISSUER" "$image" > "$scratch/$type.verified.json"
  jq -es --arg type "$type" --slurpfile expected "$predicate" '
    def normalized:
      if $type == "slsaprovenance1" and .buildDefinition.internalParameters == {} then
        del(.buildDefinition.internalParameters)
      else . end;
    any(.[]; (.payload | @base64d | fromjson | .predicate | normalized) == ($expected[0] | normalized))
  ' "$scratch/$type.verified.json" >/dev/null || {
    echo "verified $type attestation does not match the generated predicate" >&2
    exit 1
  }
}

provenance_predicate="$evidence_directory/image.provenance.json"
cosign attest --yes --type slsaprovenance1 --predicate "$provenance_predicate" "$image"
verify_image_attestation slsaprovenance1 "$provenance_predicate"
cyclonedx_predicate="$evidence_directory/image.cyclonedx.json"
cosign attest --yes --type cyclonedx --predicate "$cyclonedx_predicate" "$image"
verify_image_attestation cyclonedx "$cyclonedx_predicate"

echo "Local release evidence prepared for $RELEASE_TAG at $source_sha ($archive_count platform archives)."
echo "No tag, release, archive, or image was built, uploaded, or promoted by this script."
