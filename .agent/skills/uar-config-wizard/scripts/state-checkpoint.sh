#!/usr/bin/env bash
set -euo pipefail

# state-checkpoint.sh — Save mid-phase snapshot of session state
# Usage: state-checkpoint.sh <session_name> <phase_name> <event>

SESSION_NAME="${1:-default}"
PHASE_NAME="${2:-unknown}"
EVENT="${3:-checkpoint}"
STATE_ROOT="${CONFIG_WIZARD_STATE_DIR:-.config-wizard}"

SESSION_DIR="$STATE_ROOT/sessions/$SESSION_NAME"
STATE_FILE="$SESSION_DIR/state.json"
CHECKPOINT_DIR="$SESSION_DIR/checkpoints"

[ -d "$CHECKPOINT_DIR" ] || mkdir -p "$CHECKPOINT_DIR"

if [ ! -f "$STATE_FILE" ]; then
  echo "ERROR: state file not found: $STATE_FILE" >&2
  exit 1
fi

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
CHECKPOINT_FILE="$CHECKPOINT_DIR/${PHASE_NAME}-${EVENT}-$(date +%s).json"

# Copy current state as checkpoint
cp "$STATE_FILE" "$CHECKPOINT_FILE"

# Update state: mark phase complete + update timestamp
python3 - <<PYEOF
import json

with open("$STATE_FILE") as f:
    state = json.load(f)

state["current_phase"] = "$PHASE_NAME"
state["updated_at"] = "$TIMESTAMP"

phases = state.get("phases_complete", [])
if "$PHASE_NAME" not in phases and "$EVENT" == "phase_complete":
    phases.append("$PHASE_NAME")
state["phases_complete"] = phases

with open("$STATE_FILE", "w") as f:
    json.dump(state, f, indent=2)

print(f"CHECKPOINT:$CHECKPOINT_FILE")
PYEOF
