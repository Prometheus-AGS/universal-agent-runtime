#!/usr/bin/env bash
set -euo pipefail

# state-init.sh — Initialize or resume a config wizard session
# Usage: state-init.sh <session_name> <mode>
# Outputs: session state directory path

SESSION_NAME="${1:-default}"
MODE="${2:-wizard}"
STATE_ROOT="${CONFIG_WIZARD_STATE_DIR:-.config-wizard}"

REGISTRY="$STATE_ROOT/registry.json"
SESSION_DIR="$STATE_ROOT/sessions/$SESSION_NAME"
STATE_FILE="$SESSION_DIR/state.json"

# Create state root if needed
mkdir -p "$SESSION_DIR/checkpoints" "$SESSION_DIR/output" "$SESSION_DIR/history"

# Check for existing session
if [ -f "$STATE_FILE" ]; then
  existing_status=$(python3 -c "import json,sys; d=json.load(open('$STATE_FILE')); print(d.get('status','unknown'))" 2>/dev/null || echo "unknown")
  if [ "$existing_status" = "active" ] || [ "$existing_status" = "paused" ]; then
    echo "RESUME:$SESSION_DIR" >&1
    exit 0
  fi
fi

# Initialize new session
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
cat > "$STATE_FILE" <<EOF
{
  "session_name": "$SESSION_NAME",
  "mode": "$MODE",
  "status": "active",
  "current_phase": "init",
  "phases_complete": [],
  "wizard_answers": {},
  "model_selection": null,
  "output_files": [],
  "candle_vllm_url": null,
  "created_at": "$TIMESTAMP",
  "updated_at": "$TIMESTAMP"
}
EOF

# Update registry
if [ ! -f "$REGISTRY" ]; then
  echo '{"sessions":{}}' > "$REGISTRY"
fi
python3 - <<PYEOF
import json, sys
registry_path = "$REGISTRY"
with open(registry_path) as f:
    r = json.load(f)
r["sessions"]["$SESSION_NAME"] = {
    "path": "$SESSION_DIR",
    "mode": "$MODE",
    "created_at": "$TIMESTAMP"
}
with open(registry_path, "w") as f:
    json.dump(r, f, indent=2)
PYEOF

echo "INIT:$SESSION_DIR"
