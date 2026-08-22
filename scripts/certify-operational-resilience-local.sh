#!/usr/bin/env bash
set -euo pipefail

mode="${1:-certify}"
case "$mode" in
  preflight)
    soak_duration_seconds="${UAR_SOAK_DURATION_SECONDS:-60}"
    ;;
  certify)
    soak_duration_seconds="${UAR_SOAK_DURATION_SECONDS:-10800}"
    ;;
  *)
    echo "usage: $0 [preflight|certify]" >&2
    exit 2
    ;;
esac

[[ "$soak_duration_seconds" =~ ^[0-9]+$ ]] || {
  echo "UAR_SOAK_DURATION_SECONDS must be a non-negative integer" >&2
  exit 2
}
if [[ "$mode" == certify && "$soak_duration_seconds" -lt 10800 ]]; then
  echo "certification requires UAR_SOAK_DURATION_SECONDS >= 10800" >&2
  exit 2
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
cd "$repo_root"

for command in cargo docker git jq node pnpm sha256sum tar; do
  command -v "$command" >/dev/null || {
    echo "missing command: $command" >&2
    exit 1
  }
done
docker info >/dev/null

source_sha="$(git rev-parse HEAD)"
[[ "$source_sha" =~ ^[0-9a-f]{40}$ ]]
if [[ -n "$(git status --porcelain)" ]]; then
  echo "local certification requires a clean checkout" >&2
  exit 1
fi
if git submodule status --recursive | grep -Eq '^[-+U]'; then
  echo "local certification requires every recursive submodule at its committed pin" >&2
  exit 1
fi

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) platform="macos-arm64" ;;
  Darwin-x86_64) platform="macos-x64" ;;
  Linux-x86_64) platform="linux-x64" ;;
  Linux-aarch64 | Linux-arm64) platform="linux-arm64" ;;
  *)
    echo "unsupported local certification host: $(uname -s)-$(uname -m)" >&2
    exit 1
    ;;
esac

candidate_tag="${UAR_CANDIDATE_TAG:-operational-resilience-${source_sha:0:12}}"
artifact_dir="${UAR_LOCAL_ARTIFACT_DIR:-target/operational-resilience-candidate}"
results_dir="${UAR_RESILIENCE_RESULTS_DIR:-target/resilience-certification}"
image="${UAR_CANDIDATE_IMAGE:-uar-operational-resilience:${source_sha:0:12}}"
target_dir="${CARGO_TARGET_DIR:-target}"
package_name="universal-agent-runtime-$platform"
archive_name="$package_name.tar.gz"
package_stage="$(mktemp -d)"
package_dir="$package_stage/$package_name"

cleanup() {
  rm -rf -- "$package_stage"
}
trap cleanup EXIT

mkdir -p "$artifact_dir" "$results_dir" "$package_dir/skills"

build_started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
pnpm install --frozen-lockfile
pnpm -C frontend install --frozen-lockfile
(
  cd frontend/packages/prometheus-entity-management
  scripts/verify-pnpm-consumer-compatibility.sh
)
pnpm build
node scripts/validate-static-bundle.mjs static
cargo build --locked --release --no-default-features \
  --bin universal-agent-runtime --features server-full

cp "$target_dir/release/universal-agent-runtime" "$package_dir/"
cp -R static "$package_dir/static"
cp -R crates/prometheus-skill-system/skills "$package_dir/skills/builtin"
cp -R src/uar/runtime/matching/models "$package_dir/models"
cp README.md LICENSE example.config.yaml "$package_dir/"
tar -czf "$artifact_dir/$archive_name" -C "$package_stage" "$package_name"

docker build -t "$image" .
build_finished_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cat > "$results_dir/candidate-build.json" <<JSON
{"schema_version":1,"source_sha":"$source_sha","candidate_tag":"$candidate_tag","platform":"$platform","archive":"$archive_name","archive_sha256":"$(sha256sum "$artifact_dir/$archive_name" | cut -d' ' -f1)","image":"$image","mode":"$mode","configured_soak_duration_seconds":$soak_duration_seconds,"started_at":"$build_started_at","finished_at":"$build_finished_at"}
JSON

UAR_INSTALLED_ARTIFACT_DIR="$artifact_dir" \
UAR_CANDIDATE_ARCHIVE_GLOB="$archive_name" \
UAR_CANDIDATE_IMAGE="$image" \
UAR_CANDIDATE_SOURCE_SHA="$source_sha" \
UAR_CANDIDATE_TAG="$candidate_tag" \
UAR_SOAK_DURATION_SECONDS="$soak_duration_seconds" \
UAR_RESILIENCE_RESULTS_DIR="$results_dir" \
  scripts/certify-operational-resilience.sh

node scripts/validate-candidate-certification.mjs "$results_dir/installed-runtime"
printf '%s\n' \
  "LOCAL_OPERATIONAL_RESILIENCE_PASS source_sha=$source_sha mode=$mode duration=$soak_duration_seconds results=$results_dir"
