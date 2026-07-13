#!/usr/bin/env bash
# scripts/ci-grep-gates.sh
#
# Architectural invariants guarding the entity-migration project. Run locally
# before pushing; CI runs the same script.
#
# See docs/migration-stale-data-audit.md → "CI gates (enforced)" for the
# canonical contract.
#
# Exit status: 0 = all gates pass, 1 = at least one gate matched a pattern
# it shouldn't.

set -uo pipefail
status=0

# `check_grep_empty <label> <pattern> <path>` — fails if the pattern matches
# anywhere under <path>. Surfaces matches with line numbers so the CI log
# tells contributors exactly where to look.
check_grep_empty() {
  local label="$1"
  local pattern="$2"
  local path="$3"
  if git grep -nE "$pattern" -- "$path" >/dev/null 2>&1; then
    echo "❌ $label"
    git grep -nE "$pattern" -- "$path" | sed 's/^/    /'
    status=1
  else
    echo "✅ $label"
  fi
}

echo "=== Architectural invariants ==="
check_grep_empty "useGraphBridge retired"    "useGraphBridge"    "frontend/src/"
if node scripts/check-frontend-boundaries.mjs; then
  echo "✅ frontend dependency direction"
else
  echo "❌ frontend dependency direction"
  status=1
fi

echo
echo "=== Aesthetic contract (admin surface only) ==="
check_grep_empty "no banned fonts in admin"  "\\b(Inter|Roboto|Arial|Space Grotesk)\\b" "frontend/src/admin/"
check_grep_empty "no outline:none in admin"  "outline:\\s*none"                   "frontend/src/admin/"

echo
if [[ "$status" -eq 0 ]]; then
  echo "All CI grep gates passed."
else
  echo "One or more CI grep gates failed — see matches above."
fi

exit "$status"
