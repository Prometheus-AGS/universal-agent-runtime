#!/usr/bin/env bash
# Stop — hold completion until a pending review passes the sycophancy gate.
#
# Fires only when .prometheus/.review-pending exists. Something must create
# that marker (a phase-completion step, or the operator) or this hook is inert
# by design — a Stop hook that fires on every turn is a tax, not a gate.
#
# A first review failure may request a correction. A continuation can hand
# off honestly with the marker still pending. This is not VLBV acceptance.

set -uo pipefail

root="${CLAUDE_PROJECT_DIR:-$PWD}"
marker="$root/.prometheus/.review-pending"
[[ -f "$marker" ]] || exit 0
input="$(cat 2>/dev/null || true)"
# If the harness does not identify the Stop state, do not create a retry loop.
stop_active="$(printf '%s' "$input" | jq -er '.stop_hook_active | if type=="boolean" then tostring else error("missing Stop flag") end' 2>/dev/null || true)"
if [[ "$stop_active" != false ]]; then
  printf '%s\n' '{"systemMessage":"Review is still pending. Report that in the handoff; no phase acceptance was granted."}'
  exit 0
fi

# Locate the checker. Absent binary degrades to a warning, never a hard block.
checker=""
for c in \
  "$root/scripts/check-findings-sycophancy.sh" \
  "$root/.claude/hooks/check-findings-sycophancy.sh" \
  "$(command -v sycophancy-correction 2>/dev/null || true)"
do
  [[ -n "$c" && -x "$c" ]] && { checker="$c"; break; }
done

if [[ -z "$checker" ]]; then
  printf '%s\n' '{"systemMessage":"Review not verified: no checker is available. The pending review was preserved."}'
  exit 0
fi

err="$(mktemp 2>/dev/null || echo /tmp/syco.$$)"
if "$checker" >"$err" 2>&1; then
  rm -f "$marker" "$err" 2>/dev/null || true
  exit 0
fi

{
  echo "ANTI-SYCOPHANCY GATE — the pending review did not pass."
  echo
  sed -n '1,40p' "$err"
  echo
  echo "Fix the findings. Do not delete the marker to get past this."
  echo "A reflection leads with the delta, not with what worked."
} >&2
rm -f "$err" 2>/dev/null || true
exit 2
