#!/usr/bin/env bash
set -euo pipefail

results_dir="${UAR_RESILIENCE_RESULTS_DIR:-target/resilience-certification}"
mkdir -p "$results_dir"
started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
log="$results_dir/test.log"

set +e
cargo test --test operational_resilience -- --nocapture 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
set -e

outcome=passed
if [[ $status -ne 0 ]]; then outcome=failed; fi
cat > "$results_dir/results.json" <<JSON
{"schema_version":1,"suite":"operational-resilience","started_at":"$started","outcome":"$outcome","exit_code":$status,"thresholds":{"parallel_runs":100,"p95_ms":250,"duplicate_events":0,"test_timeout_seconds":60}}
JSON
exit "$status"
