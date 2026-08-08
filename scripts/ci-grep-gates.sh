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
check_grep_empty "no Radix icon imports"     "@radix-ui/react-icons" "frontend/src/"
check_grep_empty "no direct radix-ui imports" "from[[:space:]]+['\"]radix-ui['\"]" "frontend/src/"
check_grep_empty "no Radix icon dependency"  "\"@radix-ui/react-icons\"[[:space:]]*:" "frontend/package.json"
check_grep_empty "no Radix icons in frontend lockfile" "@radix-ui/react-icons" "frontend/pnpm-lock.yaml"
check_grep_empty "no Radix icons in root lockfile" "@radix-ui/react-icons" "pnpm-lock.yaml"
if node scripts/check-frontend-boundaries.mjs && node scripts/test-frontend-boundaries-negative.mjs; then
  echo "✅ frontend dependency direction"
else
  echo "❌ frontend dependency direction"
  status=1
fi
if node scripts/check-platform-adapters.mjs && node scripts/test-platform-adapters-negative.mjs; then
  echo "✅ platform adapter ownership"
else
  echo "❌ platform adapter ownership"
  status=1
fi
if node scripts/check-flat2-style.mjs && node scripts/test-flat2-style-negative.mjs; then
  echo "✅ Flat 2.0 style and filename contract"
else
  echo "❌ Flat 2.0 style and filename contract"
  status=1
fi
if node scripts/check-hsl-token-codemod.mjs && node scripts/test-hsl-token-codemod-negative.mjs; then
  echo "✅ semantic color token migration"
else
  echo "❌ semantic color token migration"
  status=1
fi
if node scripts/check-storybook-a11y-suppressions.mjs && node scripts/test-storybook-a11y-suppressions-negative.mjs; then
  echo "✅ fail-closed Storybook accessibility"
else
  echo "❌ fail-closed Storybook accessibility"
  status=1
fi

echo
if [[ "$status" -eq 0 ]]; then
  echo "All CI grep gates passed."
else
  echo "One or more CI grep gates failed — see matches above."
fi

exit "$status"
