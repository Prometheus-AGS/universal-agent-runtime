#!/usr/bin/env bash
# scripts/live-integration.sh — run the live integration tier
# (proxy-integration-gate) against the operator's local OpenAI-compatible
# proxy, or fall back to the recorded (stub-LLM) backend.
#
# Usage:
#   scripts/live-integration.sh                        # health-check the proxy, run live
#   scripts/live-integration.sh --allow-recorded-fallback   # CI default: skip health
#                                                            # check, run recorded if the
#                                                            # proxy isn't reachable
#
# This is the "feature correctness" gate (design.md, proxy-integration-gate):
# does streaming/tool-calls/memory/RAG/credentials actually work end-to-end?
# It is distinct from the eval harness (evals/) which gates model *quality*.
#
# On proxy health-check failure (and without --allow-recorded-fallback), exits
# non-zero with the two-step remediation instead of letting the first test
# case fail with an opaque connection/401 error.

set -euo pipefail

die() { printf 'live-integration: %s\n' "$*" >&2; exit 1; }

PROXY_URL="${UAR_LIVE_PROXY_URL:-http://127.0.0.1:8181/v1}"
ALLOW_RECORDED_FALLBACK=0

for arg in "$@"; do
  case "$arg" in
    --allow-recorded-fallback) ALLOW_RECORDED_FALLBACK=1 ;;
    *) die "unknown argument: $arg (usage: $0 [--allow-recorded-fallback])" ;;
  esac
done

remediate() {
  cat >&2 <<EOF

live-integration: the local OpenAI-compatible proxy at ${PROXY_URL} is not
responding. This tier needs it for --backend=live runs.

To fix:
  1. Re-authenticate the Codex-backed proxy (run \`codex login\` or equivalent
     for your Codex CLI, if the token has expired).
  2. Restart the proxy service:
       launchctl kickstart -k gui/501/ai.prometheus.openai-proxy

Then re-run this script. Or pass --allow-recorded-fallback to run the
recorded (in-process stub) backend instead.

EOF
}

health_check() {
  curl -fsS -m 5 -o /dev/null "${PROXY_URL}/models"
}

if health_check; then
  echo "live-integration: proxy healthy at ${PROXY_URL} — running live backend"
  export UAR_LIVE_INTEGRATION_BACKEND=live
elif [[ "$ALLOW_RECORDED_FALLBACK" -eq 1 ]]; then
  echo "live-integration: proxy unreachable at ${PROXY_URL} — falling back to recorded backend (--allow-recorded-fallback)"
  export UAR_LIVE_INTEGRATION_BACKEND=recorded
else
  remediate
  exit 1
fi

exec cargo test --test integration live::
