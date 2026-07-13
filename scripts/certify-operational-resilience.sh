#!/usr/bin/env bash
set -euo pipefail

results_dir="${UAR_RESILIENCE_RESULTS_DIR:-target/resilience-certification}"
artifact_dir="${UAR_INSTALLED_ARTIFACT_DIR:-}"
mkdir -p "$results_dir"
started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
log="$results_dir/test.log"

set +e
cargo test --test operational_resilience -- --nocapture 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
set -e

outcome=passed
if [[ $status -ne 0 ]]; then outcome=failed; fi

installed_result=null
if [[ $status -eq 0 && -n "$artifact_dir" ]]; then
  installed_dir="$results_dir/installed-runtime"
  set +e
  UAR_SOAK_DURATION_SECONDS="${UAR_SOAK_DURATION_SECONDS:-60}" \
  UAR_CANDIDATE_SOURCE_SHA="${UAR_CANDIDATE_SOURCE_SHA:-$(git rev-parse HEAD)}" \
  UAR_CANDIDATE_TAG="${UAR_CANDIDATE_TAG:-operational-resilience-${GITHUB_RUN_ID:-local}}" \
    scripts/certify-release-candidate.sh "$artifact_dir" "$installed_dir" \
      2>&1 | tee "$results_dir/installed-runtime.log"
  installed_status=${PIPESTATUS[0]}
  set -e
  if [[ $installed_status -ne 0 ]]; then
    outcome=failed
    status=$installed_status
  elif [[ -f "$installed_dir/results.json" ]]; then
    installed_result="installed-runtime/results.json"
  fi
fi

finished="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
cat > "$results_dir/results.json" <<JSON
{"schema_version":2,"suite":"operational-resilience","started_at":"$started","finished_at":"$finished","source_sha":"${UAR_CANDIDATE_SOURCE_SHA:-$(git rev-parse HEAD)}","candidate_tag":"${UAR_CANDIDATE_TAG:-operational-resilience-${GITHUB_RUN_ID:-local}}","outcome":"$outcome","exit_code":$status,"deterministic_tests":"test.log","installed_runtime_result":$(if [[ "$installed_result" == null ]]; then printf null; else printf '"%s"' "$installed_result"; fi)} }
JSON
exit "$status"
