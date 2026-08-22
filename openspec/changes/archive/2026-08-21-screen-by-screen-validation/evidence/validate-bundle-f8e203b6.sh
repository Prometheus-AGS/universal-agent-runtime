#!/usr/bin/env bash
set -euo pipefail

bundle="${1:-docs/certifications/product-screens/f8e203b6}"
manifest="$bundle/manifest.json"
expected_sha="f8e203b64462681597155f83660a9f35e03efa4c"
actual_sha="$(jq -r '.git_sha' "$manifest")"
git cat-file -e "$expected_sha^{commit}"
test "$actual_sha" = "$expected_sha"
echo "GIT_SHA_MATCH=$actual_sha"

source_dir="$(jq -r '.module_fingerprint_source' "$manifest")"
module_fingerprint="sha256:$(
  git ls-tree -r --name-only "$expected_sha" -- "$source_dir" \
    | LC_ALL=C sort \
    | while IFS= read -r file; do
        git show "$expected_sha:$file" | shasum -a 256 | awk '{print $1}'
      done \
    | shasum -a 256 \
    | awk '{print $1}'
)"
test "$module_fingerprint" = "$(jq -r '.module_fingerprint' "$manifest")"
echo "MODULE_FINGERPRINT_MATCH=$module_fingerprint"

mapfile -t source_paths < <(jq -r '.git_tree_fingerprint.source_paths[]' "$manifest")
tree_fingerprint="$(
  git ls-tree -r --full-tree "$expected_sha" -- "${source_paths[@]}" \
    | LC_ALL=C sort \
    | shasum -a 256 \
    | awk '{print $1}'
)"
test "$tree_fingerprint" = "$(jq -r '.git_tree_fingerprint.sha256' "$manifest")"
echo "GIT_TREE_FINGERPRINT_MATCH=$tree_fingerprint"

artifact_count=0
while IFS=$'\t' read -r path expected_hash expected_bytes; do
  file="$bundle/$path"
  test -f "$file"
  actual_hash="$(shasum -a 256 "$file" | awk '{print $1}')"
  actual_bytes="$(wc -c < "$file" | tr -d ' ')"
  test "$actual_hash" = "$expected_hash"
  test "$actual_bytes" = "$expected_bytes"
  artifact_count=$((artifact_count + 1))
done < <(jq -r '.artifacts[] | [.path, .sha256, (.bytes | tostring)] | @tsv' "$manifest")
test "$artifact_count" -eq 54
echo "ARTIFACTS_MATCH=$artifact_count"

duplicate_paths="$(
  jq '(.artifacts | map(.path) | length) - (.artifacts | map(.path) | unique | length)' "$manifest"
)"
test "$duplicate_paths" -eq 0
echo "DUPLICATE_ARTIFACT_PATHS=$duplicate_paths"

video_count=0
declare -A video_hashes=()
while IFS= read -r video; do
  codec="$(ffprobe -v error -select_streams v:0 -show_entries stream=codec_name -of default=nw=1:nk=1 "$video")"
  duration="$(ffprobe -v error -show_entries format=duration -of default=nw=1:nk=1 "$video")"
  test "$codec" = "h264"
  awk -v duration="$duration" 'BEGIN { exit !(duration > 0) }'
  hash="$(shasum -a 256 "$video" | awk '{print $1}')"
  test -z "${video_hashes[$hash]:-}"
  video_hashes["$hash"]=1
  video_count=$((video_count + 1))
done < <(find "$bundle/videos" -type f -name '*.mp4' | LC_ALL=C sort)
test "$video_count" -eq 32
test "${#video_hashes[@]}" -eq 32
echo "VIDEOS_H264_POSITIVE_DURATION=$video_count"
echo "UNIQUE_VIDEO_HASHES=${#video_hashes[@]}"

screenshot_count="$(find "$bundle/screenshots" -type f -name '*.png' | wc -l | tr -d ' ')"
test "$screenshot_count" -eq 20
echo "SCREENSHOT_COUNT=$screenshot_count"

scenario_count="$(jq '[.[].elements[]] | length' "$bundle/cucumber-report.json")"
failed_count="$(jq '[.[].elements[].steps[].result.status | select(. != "passed")] | length' "$bundle/cucumber-report.json")"
test "$scenario_count" -eq 32
test "$failed_count" -eq 0
echo "CUCUMBER_SCENARIOS=$scenario_count"
echo "CUCUMBER_FAILED=$failed_count"

python3 - "$bundle/report.html" "$manifest" <<'PY'
import html
import json
import sys

report = open(sys.argv[1], encoding="utf-8").read()
manifest = json.load(open(sys.argv[2], encoding="utf-8"))
needle = html.escape(json.dumps(manifest, indent=2))
if needle not in report:
    raise SystemExit("finalized manifest is absent from report")
PY
echo "REPORT_FINALIZED_MANIFEST_PRESENT"

tampered="$(mktemp)"
trap 'rm -f "$tampered"' EXIT
cp "$bundle/cucumber-report.json" "$tampered"
printf 'x' >> "$tampered"
expected_cucumber_hash="$(jq -r '.artifacts[] | select(.path == "cucumber-report.json") | .sha256' "$manifest")"
tampered_hash="$(shasum -a 256 "$tampered" | awk '{print $1}')"
test "$tampered_hash" != "$expected_cucumber_hash"
echo "TAMPER_CONTROL_REJECTED expected=$expected_cucumber_hash mutated=$tampered_hash"
