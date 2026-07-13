#!/usr/bin/env bash
set -euo pipefail

# Promote certified RC bytes to GA without rebuilding them.
# Dry-run is the default. Mutation requires --execute plus an exact confirmation
# and, when replacing a stale GA tag, its expected current commit.

candidate="${1:?candidate tag is required (for example v1.0.0-rc.3)}"
ga="${2:?GA tag is required (for example v1.0.0)}"
mode="${3:---dry-run}"
repo="${GITHUB_REPOSITORY:-Prometheus-AGS/universal-agent-runtime}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/assets" "$work/certification" "$work/expected-assets" "$work/tag-verification"

for command in git gh jq node sha256sum tar unzip; do command -v "$command" >/dev/null || { echo "missing command: $command" >&2; exit 1; }; done

download_authenticated_set() {
  local tag="$1"
  local directory="$2"
  local index="$3"
  local signer="$4"
  gh release download "$tag" --repo "$repo" --dir "$directory" --pattern "$index"
  gh attestation verify "$directory/$index" --repo "$repo" --signer-workflow "$signer"
  while read -r _ asset; do
    [[ "$asset" =~ ^[A-Za-z0-9._@+-]+$ ]] || { echo "unsafe authenticated asset name: $asset" >&2; return 1; }
    gh release download "$tag" --repo "$repo" --dir "$directory" --pattern "$asset"
  done <"$directory/$index"
  (cd "$directory" && sha256sum --check "$index")
}

asset_inventory() {
  local directory="$1"
  local file
  for file in "$directory"/*; do
    [[ -f "$file" ]] || continue
    printf '%s  %s\n' "$(sha256sum "$file" | cut -d' ' -f1)" "$(basename "$file")"
  done | sort
}
if [[ "$candidate" =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+)-rc\.([0-9]+)$ ]]; then
  candidate_version="${BASH_REMATCH[1]}.${BASH_REMATCH[2]}.${BASH_REMATCH[3]}"
else
  echo "candidate tag must be v<semver>-rc.<n>" >&2
  exit 1
fi
if [[ "$ga" =~ ^v([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
  ga_version="${BASH_REMATCH[1]}.${BASH_REMATCH[2]}.${BASH_REMATCH[3]}"
else
  echo "GA tag must be v<semver> without a prerelease suffix" >&2
  exit 1
fi
[[ "$candidate_version" == "$ga_version" ]] || {
  echo "candidate $candidate cannot be promoted as a different GA version $ga" >&2
  exit 1
}

origin_url="$(git -C "$root" remote get-url origin)"
git -C "$work/tag-verification" init -q
git -C "$work/tag-verification" fetch -q "$origin_url" \
  "refs/tags/${candidate}:refs/tags/${candidate}"
git -C "$work/tag-verification" verify-tag "$candidate"
candidate_sha="$(git -C "$work/tag-verification" rev-parse "${candidate}^{}")"
download_authenticated_set "$candidate" "$work/assets" SHA256SUMS \
  "${repo}/.github/workflows/supply-chain.yml"
manifest="$work/assets/release-manifest.json"
[[ -f "$manifest" ]] || { echo "candidate has no release-manifest.json" >&2; exit 1; }
promotion="$work/assets/promotion.json"
[[ -f "$promotion" ]] || { echo "candidate has no authenticated promotion.json" >&2; exit 1; }
checksums="$work/assets/SHA256SUMS"
[[ -f "$checksums" ]] || { echo "candidate has no SHA256SUMS trust root" >&2; exit 1; }
node "$root/scripts/validate-release-manifest.mjs" "$manifest"

download_authenticated_set "$candidate" "$work/certification" \
  CANDIDATE_CERTIFICATION_SHA256SUMS \
  "${repo}/.github/workflows/candidate-certification.yml"
node "$root/scripts/validate-candidate-certification-bundle.mjs" \
  "$work/certification" "$manifest"
certification_archive="$(jq -r .archive.name "$work/certification/candidate-certification-manifest.json")"
mkdir -p "$work/certification-results"
tar -xzf "$work/certification/$certification_archive" -C "$work/certification-results"
node "$root/scripts/validate-candidate-certification.mjs" \
  "$work/certification-results/release-candidate-certification"

manifest_release="$(jq -r .release "$manifest")"
manifest_sha="$(jq -r .source.sha "$manifest")"
[[ "$manifest_release" == "$candidate" ]] || { echo "manifest release does not match candidate tag" >&2; exit 1; }
[[ "$manifest_sha" == "$candidate_sha" ]] || { echo "manifest source does not match candidate tag" >&2; exit 1; }

promotion_schema="$(jq -r .schema_version "$promotion")"
promotion_candidate="$(jq -r .candidate "$promotion")"
promotion_ga="$(jq -r .ga "$promotion")"
promotion_sha="$(jq -r .source_sha "$promotion")"
promotion_image="$(jq -r .image "$promotion")"
promotion_rebuild="$(jq -r .rebuild "$promotion")"
promotion_superseded_sha="$(jq -r .superseded_ga_sha "$promotion")"
image_ref="$(jq -r .image.reference "$manifest")"
image_digest="$(jq -r .image.digest "$manifest")"
[[ "$promotion_schema" == 1 ]] || { echo "unsupported promotion metadata schema" >&2; exit 1; }
[[ "$promotion_candidate" == "$candidate" ]] || { echo "promotion metadata candidate mismatch" >&2; exit 1; }
[[ "$promotion_ga" == "$ga" ]] || { echo "promotion metadata GA mismatch" >&2; exit 1; }
[[ "$promotion_sha" == "$candidate_sha" ]] || { echo "promotion metadata source mismatch" >&2; exit 1; }
[[ "$promotion_image" == "$image_ref@$image_digest" ]] || { echo "promotion metadata image mismatch" >&2; exit 1; }
[[ "$promotion_rebuild" == false ]] || { echo "promotion metadata permits a rebuild" >&2; exit 1; }
[[ "$promotion_superseded_sha" == null || "$promotion_superseded_sha" =~ ^[0-9a-f]{40}$ ]] || {
  echo "promotion metadata has an invalid superseded GA SHA" >&2
  exit 1
}

# This local contract gate prevents the GA tag pushed below from matching the
# archive-build workflow. Candidate bytes must be reused, never rebuilt.
node "$root/scripts/validate-release-workflow.mjs"

remote_line="$(git -C "$root" ls-remote origin "refs/tags/${ga}^{}" | head -1)"
if [[ -z "$remote_line" ]]; then remote_line="$(git -C "$root" ls-remote origin "refs/tags/${ga}" | head -1)"; fi
existing_sha="${remote_line%%[[:space:]]*}"
raw_existing_line="$(git -C "$root" ls-remote origin "refs/tags/${ga}" | head -1)"
raw_existing_ref="${raw_existing_line%%[[:space:]]*}"
actual_existing_sha="${existing_sha:-null}"
case "$actual_existing_sha" in
  "$promotion_superseded_sha") tag_state=not-started ;;
  "$candidate_sha")
    tag_state=already-promoted
    git -C "$work/tag-verification" fetch -q -f "$origin_url" "refs/tags/${ga}:refs/tags/${ga}"
    git -C "$work/tag-verification" verify-tag "$ga"
    [[ "$(git -C "$work/tag-verification" rev-parse "${ga}^{}")" == "$candidate_sha" ]] || {
      echo "signed GA tag does not resolve to the candidate commit" >&2
      exit 1
    }
    ;;
  *)
    echo "GA tag state changed: authenticated pre-state is $promotion_superseded_sha, resumable state is $candidate_sha, origin has $actual_existing_sha" >&2
    exit 1
    ;;
esac

cp "$work/assets"/* "$work/expected-assets/"
cp "$work/certification"/* "$work/expected-assets/"

echo "Candidate: $candidate ($candidate_sha)"
echo "GA:        $ga (${existing_sha:-unpublished}; $tag_state)"
echo "Image:     $image_ref@$image_digest"
asset_count="$(( $(asset_inventory "$work/assets" | wc -l | tr -d ' ') + $(asset_inventory "$work/certification" | wc -l | tr -d ' ') ))"
echo "Assets:    $asset_count authenticated candidate files, copied unchanged"

if [[ "$mode" != "--execute" ]]; then
  echo "Dry run complete; no tags, releases, images, or remote state changed."
  exit 0
fi

[[ "${UAR_CONFIRM_GA_PROMOTION:-}" == "${candidate}->${ga}" ]] || {
  echo "set UAR_CONFIRM_GA_PROMOTION=${candidate}->${ga} to execute" >&2
  exit 1
}
if [[ "$tag_state" == not-started && "$promotion_superseded_sha" != null ]]; then
  [[ "${UAR_EXPECTED_EXISTING_GA_SHA:-}" == "$promotion_superseded_sha" ]] || {
    echo "set UAR_EXPECTED_EXISTING_GA_SHA=$promotion_superseded_sha to authorize replacement of the audited stale tag" >&2
    exit 1
  }
fi
command -v docker >/dev/null || { echo "docker is required to promote the image manifest" >&2; exit 1; }
if [[ "$tag_state" == not-started ]]; then
  git -C "$work/tag-verification" tag -s -f "$ga" "$candidate_sha" \
    -m "Universal Agent Runtime ${ga#v}; promoted unchanged from $candidate"
  git -C "$work/tag-verification" verify-tag "$ga"
  git -C "$work/tag-verification" push "$origin_url" "refs/tags/$ga:refs/tags/$ga" \
    --force-with-lease="refs/tags/$ga:${raw_existing_ref}"
  git -C "$work/tag-verification" fetch -q -f "$origin_url" "refs/tags/${ga}:refs/tags/${ga}"
  git -C "$work/tag-verification" verify-tag "$ga"
  [[ "$(git -C "$work/tag-verification" rev-parse "${ga}^{}")" == "$candidate_sha" ]] || {
    echo "published GA tag does not resolve to the candidate commit" >&2
    exit 1
  }
fi

# This creates a second tag for the already-certified OCI manifest digest; it
# does not invoke a Docker build. Cosign verification remains digest-bound.
ga_image="${image_ref}:${ga#v}"
current_image_digest="$(docker buildx imagetools inspect "$ga_image" 2>/dev/null | awk '$1 == "Digest:" { print $2; exit }' || true)"
if [[ "$current_image_digest" == "$image_digest" ]]; then
  echo "Image alias already resolves to the certified digest; skipping."
else
  docker buildx imagetools create --tag "$ga_image" "${image_ref}@${image_digest}"
  published_image_digest="$(docker buildx imagetools inspect "$ga_image" | awk '$1 == "Digest:" { print $2; exit }')"
  [[ "$published_image_digest" == "$image_digest" ]] || { echo "GA image alias digest mismatch after publication" >&2; exit 1; }
fi

release_state=absent
if gh release view "$ga" --repo "$repo" >/dev/null 2>&1; then
  mkdir -p "$work/current-release"
  gh release download "$ga" --repo "$repo" --dir "$work/current-release" 2>/dev/null || true
  release_flags="$(gh release view "$ga" --repo "$repo" --json isDraft,isPrerelease --jq '[.isDraft,.isPrerelease] | @tsv')"
  if [[ "$release_flags" == $'false\tfalse' && "$(asset_inventory "$work/current-release")" == "$(asset_inventory "$work/expected-assets")" ]]; then
    release_state=exact
  else
    release_state=mismatch
  fi
fi
case "$release_state" in
  exact)
    echo "GA release already contains the exact authenticated asset set; skipping."
    ;;
  mismatch)
    [[ "${UAR_CONFIRM_GA_RELEASE_REPLACEMENT:-}" == "$ga" ]] || {
      echo "GA release assets differ; set UAR_CONFIRM_GA_RELEASE_REPLACEMENT=$ga to replace them" >&2
      exit 1
    }
    gh release delete "$ga" --repo "$repo" --yes
    gh release create "$ga" "$work/expected-assets"/* --repo "$repo" --verify-tag \
      --title "Universal Agent Runtime ${ga#v}" \
      --notes "Promoted unchanged from certified candidate ${candidate}. See promotion.json and release-manifest.json."
    ;;
  absent)
    gh release create "$ga" "$work/expected-assets"/* --repo "$repo" --verify-tag \
      --title "Universal Agent Runtime ${ga#v}" \
      --notes "Promoted unchanged from certified candidate ${candidate}. See promotion.json and release-manifest.json."
    ;;
esac

mkdir -p "$work/published-release"
gh release download "$ga" --repo "$repo" --dir "$work/published-release"
[[ "$(asset_inventory "$work/published-release")" == "$(asset_inventory "$work/expected-assets")" ]] || {
  echo "published GA release asset inventory mismatch" >&2
  exit 1
}
[[ "$(gh release view "$ga" --repo "$repo" --json isDraft,isPrerelease --jq '[.isDraft,.isPrerelease] | @tsv')" == $'false\tfalse' ]] || {
  echo "GA release is not public and stable" >&2
  exit 1
}

echo "Promoted $candidate to $ga without rebuilding release payloads."
