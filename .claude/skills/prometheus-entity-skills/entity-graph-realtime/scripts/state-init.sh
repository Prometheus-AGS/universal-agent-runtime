#!/usr/bin/env bash
set -euo pipefail

# Initialize or resume workflow state for entity-graph-realtime sessions.
# Usage: state-init.sh [workspace_root]
# State is stored under <workspace>/.entity-graph-skills/entity-graph-realtime/state.json

ROOT="${1:-.}"
STATE_DIR="${ROOT}/.entity-graph-skills/entity-graph-realtime"
STATE_FILE="${STATE_DIR}/state.json"
mkdir -p "$STATE_DIR"

NOW=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

if [[ -f "$STATE_FILE" ]]; then
  echo "Resuming existing state at ${STATE_FILE}"
  cat "$STATE_FILE"
  exit 0
fi

ORCH=$(
  cd "$ROOT" && bash "$(dirname "$0")/detect-orchestrators.sh" 2>/dev/null || echo '{}'
)

cat > "$STATE_FILE" << EOF
{
  "skill": "entity-graph-realtime",
  "created_at": "${NOW}",
  "updated_at": "${NOW}",
  "current_phase": "specify",
  "phases_completed": [],
  "orchestrators": ${ORCH},
  "realtime_spec": null,
  "plan": null,
  "artifacts": []
}
EOF

echo "Initialized ${STATE_FILE}"
cat "$STATE_FILE"
