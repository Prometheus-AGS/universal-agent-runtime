#!/usr/bin/env bash
set -euo pipefail

# Promote certified RC bytes to GA without rebuilding them.
# Dry-run is the default. Mutation requires --execute plus an exact confirmation
# and, when replacing a stale GA tag, its expected current commit.

candidate="${1:?candidate tag is required (for example v1.0.0-rc.2)}"
ga="${2:?GA tag is required (for example v1.0.0)}"
mode="${3:---dry-run}"
repo="${GITHUB_REPOSITORY:-Prometheus-AGS/universal-agent-runtime}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/assets"

for command in git gh jq node; do command -v "$command" >/dev/null || { echo "missing command: $command" >&2; exit 1; }; done
[[ "$candidate" == v*.*.*-rc.* ]] || { echo "candidate tag must be v<semver>-rc.<n>" >&2; exit 1; }
[[ "$ga" == v*.*.* && "$ga" != *-* ]] || { echo "GA tag must be v<semver> without a prerelease suffix" >&2; exit 1; }

candidate_line="$(git -C "$root" ls-remote origin "refs/tags/${candidate}^{}" | head -1)"
if [[ -z "$candidate_line" ]]; then candidate_line="$(git -C "$root" ls-remote origin "refs/tags/${candidate}" | head -1)"; fi
candidate_sha="${candidate_line%%[[:space:]]*}"
[[ -n "$candidate_sha" ]] || { echo "candidate tag does not exist on origin" >&2; exit 1; }
gh release download "$candidate" --repo "$repo" --dir "$work/assets"
manifest="$work/assets/release-manifest.json"
[[ -f "$manifest" ]] || { echo "candidate has no release-manifest.json" >&2; exit 1; }
node "$root/scripts/validate-release-manifest.mjs" "$manifest"

manifest_release="$(jq -r .release "$manifest")"
manifest_sha="$(jq -r .source.sha "$manifest")"
[[ "$manifest_release" == "$candidate" ]] || { echo "manifest release does not match candidate tag" >&2; exit 1; }
[[ "$manifest_sha" == "$candidate_sha" ]] || { echo "manifest source does not match candidate tag" >&2; exit 1; }

remote_line="$(git -C "$root" ls-remote origin "refs/tags/${ga}^{}" | head -1)"
if [[ -z "$remote_line" ]]; then remote_line="$(git -C "$root" ls-remote origin "refs/tags/${ga}" | head -1)"; fi
existing_sha="${remote_line%%[[:space:]]*}"
if [[ -n "$existing_sha" && "$existing_sha" != "$candidate_sha" ]]; then
  [[ "${UAR_EXPECTED_EXISTING_GA_SHA:-}" == "$existing_sha" ]] || {
    echo "GA tag exists at $existing_sha; set UAR_EXPECTED_EXISTING_GA_SHA to that exact commit after audit" >&2
    exit 1
  }
fi

image_ref="$(jq -r .image.reference "$manifest")"
image_digest="$(jq -r .image.digest "$manifest")"
cat >"$work/assets/promotion.json" <<JSON
{"schema_version":1,"candidate":"$candidate","ga":"$ga","source_sha":"$candidate_sha","candidate_manifest_sha256":"$(shasum -a 256 "$manifest" | cut -d' ' -f1)","image":"$image_ref@$image_digest","rebuild":false,"superseded_ga_sha":"${existing_sha:-null}"}
JSON

echo "Candidate: $candidate ($candidate_sha)"
echo "GA:        $ga (${existing_sha:-unpublished})"
echo "Image:     $image_ref@$image_digest"
echo "Assets:    $(find "$work/assets" -maxdepth 1 -type f | wc -l | tr -d ' ') exact candidate files plus promotion.json"

if [[ "$mode" != "--execute" ]]; then
  echo "Dry run complete; no tags, releases, images, or remote state changed."
  exit 0
fi

[[ "${UAR_CONFIRM_GA_PROMOTION:-}" == "${candidate}->${ga}" ]] || {
  echo "set UAR_CONFIRM_GA_PROMOTION=${candidate}->${ga} to execute" >&2
  exit 1
}
command -v docker >/dev/null || { echo "docker is required to promote the image manifest" >&2; exit 1; }
git -C "$root" fetch origin "refs/tags/${candidate}:refs/tags/${candidate}" --force

# `-s` is intentional: promotion stops unless the maintainer has configured a
# usable Git signing identity. The remote replacement is guarded by the exact
# audited old commit above.
git -C "$root" tag -s -f "$ga" "$candidate_sha" -m "Universal Agent Runtime ${ga#v}; promoted unchanged from $candidate"
if [[ -n "$existing_sha" && "$existing_sha" != "$candidate_sha" ]]; then
  gh release view "$ga" --repo "$repo" --json tagName,name,url,publishedAt >"$work/assets/superseded-release.json" 2>/dev/null || true
  gh release delete "$ga" --repo "$repo" --yes 2>/dev/null || true
  git -C "$root" push origin ":refs/tags/$ga"
fi
git -C "$root" push origin "refs/tags/$ga"

# This creates a second tag for the already-certified OCI manifest digest; it
# does not invoke a Docker build. Cosign verification remains digest-bound.
docker buildx imagetools create --tag "${image_ref}:${ga#v}" "${image_ref}@${image_digest}"
gh release create "$ga" "$work/assets"/* --repo "$repo" --verify-tag \
  --title "Universal Agent Runtime ${ga#v}" \
  --notes "Promoted unchanged from certified candidate ${candidate}. See promotion.json and release-manifest.json."

echo "Promoted $candidate to $ga without rebuilding release payloads."
