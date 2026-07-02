#!/usr/bin/env bash
# tools/live-matrix-check.sh — feature-coverage matrix presence check.
#
# Every change in the uar-next-harness phase plan that lands a user-facing
# runtime feature must add a row to tests/integration/live/MATRIX.md mapping
# its CH-## identifier to a live integration test case (plan Amendment A2.3).
#
# This script reports CH-## identifiers that appear in the phase plan's Round
# 1-4 change list but are MISSING from MATRIX.md. It is ADVISORY: it prints a
# warning and exits 0 (the workflow runs it under continue-on-error). Promote
# to gating (exit 1 on drift) once CH-01..CH-04 have each added a row.
#
# Runs identically from any tool (Codex, Claude Code, Cursor, OpenCode) — it's
# plain bash + grep with no tool-specific hooks.

set -uo pipefail

PLAN=".kbd-orchestrator/phases/uar-next-harness/plan.md"
MATRIX="tests/integration/live/MATRIX.md"

if [[ ! -f "$MATRIX" ]]; then
  echo "live-matrix-check: MATRIX.md not found at $MATRIX" >&2
  exit 0
fi
if [[ ! -f "$PLAN" ]]; then
  echo "live-matrix-check: phase plan not found at $PLAN — skipping (nothing to check against)"
  exit 0
fi

# CH-## tokens referenced anywhere in the plan (feature changes carry them).
mapfile -t plan_changes < <(grep -oE 'CH-[0-9]+[a-z]?' "$PLAN" | sort -u)

missing=()
for ch in "${plan_changes[@]}"; do
  if ! grep -q "$ch" "$MATRIX"; then
    missing+=("$ch")
  fi
done

if [[ ${#missing[@]} -eq 0 ]]; then
  echo "live-matrix-check: OK — every plan CH-## is present in MATRIX.md"
  exit 0
fi

echo "live-matrix-check: ADVISORY — the following plan changes have no MATRIX.md row yet:" >&2
for ch in "${missing[@]}"; do
  echo "  - $ch" >&2
done
echo "(Add a row in $MATRIX when the change lands a user-facing feature. Advisory only — not failing the build.)" >&2
exit 0
