#!/usr/bin/env bash
set -euo pipefail

# Initialize or resume workflow state for entity-graph-setup sessions.
# Usage: bash prometheus-entity-skills/entity-graph-setup/scripts/state-init.sh [workspace_root]
# State: <workspace>/.entity-graph-skills/entity-graph-setup/state.json

ROOT="${1:-.}"
STATE_DIR="${ROOT}/.entity-graph-skills/entity-graph-setup"
STATE_FILE="${STATE_DIR}/state.json"
mkdir -p "$STATE_DIR"

NOW=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

if [[ -f "$STATE_FILE" ]]; then
  echo "Resuming existing state at ${STATE_FILE}"
  cat "$STATE_FILE"
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ORCH=$(
  cd "$ROOT" && bash "${SCRIPT_DIR}/detect-orchestrators.sh" 2>/dev/null || echo '{}'
)

cat > "$STATE_FILE" << EOF
{
  "skill": "entity-graph-setup",
  "created_at": "${NOW}",
  "updated_at": "${NOW}",
  "current_phase": "specify",
  "phases_completed": [],
  "orchestrators": ${ORCH},
  "setup_spec": null,
  "entity_manifest": null,
  "migration_plan": null,
  "reflect_report": null,
  "artifacts": []
}
EOF

echo "Initialized ${STATE_FILE}"
cat "$STATE_FILE"
