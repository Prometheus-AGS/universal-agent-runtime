#!/usr/bin/env bash
set -euo pipefail

# state-finalize.sh — Archive a completed config wizard session
# Usage: state-finalize.sh <session_name>

SESSION_NAME="${1:-default}"
STATE_ROOT="${CONFIG_WIZARD_STATE_DIR:-.config-wizard}"

SESSION_DIR="$STATE_ROOT/sessions/$SESSION_NAME"
STATE_FILE="$SESSION_DIR/state.json"

if [ ! -f "$STATE_FILE" ]; then
  echo "WARNING: state file not found: $STATE_FILE — nothing to finalize" >&2
  exit 0
fi

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Mark session complete
python3 - <<PYEOF
import json

with open("$STATE_FILE") as f:
    state = json.load(f)

state["status"] = "complete"
state["updated_at"] = "$TIMESTAMP"

with open("$STATE_FILE", "w") as f:
    json.dump(state, f, indent=2)

output_files = state.get("output_files", [])
written = [f["filename"] for f in output_files if f.get("written", False)]
print(f"FINALIZED:$SESSION_NAME")
print(f"OUTPUT_FILES:{len(written)}")
for f in written:
    print(f"  - {f}")
PYEOF
