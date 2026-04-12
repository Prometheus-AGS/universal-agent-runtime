#!/usr/bin/env bash
# Run from the application workspace root (where .kbd-orchestrator, prisma, etc. live).
set -euo pipefail
SKILL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_DIR="${SKILL_STATE_DIR:-$SKILL_ROOT/.skill-state}"
mkdir -p "$STATE_DIR"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '%s\n' "$TS" > "$STATE_DIR/last-activate.txt"
printf '%s\n' "$SKILL_ROOT" > "$STATE_DIR/skill-root.path"
# Snapshot orchestrator availability when cwd is the project root
if [[ "${PWD:-}" != "" ]] && [[ -x "$SKILL_ROOT/scripts/detect-orchestrators.sh" ]]; then
  (cd "$PWD" && bash "$SKILL_ROOT/scripts/detect-orchestrators.sh") > "$STATE_DIR/orchestrators.json" 2>/dev/null || printf '{}\n' > "$STATE_DIR/orchestrators.json"
else
  printf '{}\n' > "$STATE_DIR/orchestrators.json"
fi
