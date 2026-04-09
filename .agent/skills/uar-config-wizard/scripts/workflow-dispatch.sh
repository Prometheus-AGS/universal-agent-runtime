#!/usr/bin/env bash
set -euo pipefail

# workflow-dispatch.sh — Lifecycle event trigger dispatcher
# Usage: workflow-dispatch.sh <session_name> <event> [phase]
# Events: phase_complete | session_complete | validation_failed | model_selected

SESSION_NAME="${1:-default}"
EVENT="${2:-unknown}"
PHASE="${3:-}"
STATE_ROOT="${CONFIG_WIZARD_STATE_DIR:-.config-wizard}"

SESSION_DIR="$STATE_ROOT/sessions/$SESSION_NAME"
STATE_FILE="$SESSION_DIR/state.json"
DISPATCH_LOG="$STATE_ROOT/dispatch.log"

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Log the dispatch event
echo "$TIMESTAMP [$SESSION_NAME] event=$EVENT phase=$PHASE" >> "$DISPATCH_LOG" 2>/dev/null || true

# Load current state safely
if [ ! -f "$STATE_FILE" ]; then
  echo "WARNING: no state file for session '$SESSION_NAME'" >&2
  exit 0
fi

case "$EVENT" in
  phase_complete)
    case "$PHASE" in
      wizard)
        # After wizard: check if model selection needed
        ROUTE=$(python3 -c "import json; d=json.load(open('$STATE_FILE')); print(d.get('wizard_answers',{}).get('route_to_model_select', False))" 2>/dev/null || echo "False")
        if [ "$ROUTE" = "True" ]; then
          echo "DISPATCH:trigger_model_select"
        else
          echo "DISPATCH:trigger_generate"
        fi
        ;;
      model_select)
        echo "DISPATCH:trigger_generate"
        ;;
      generate)
        echo "DISPATCH:trigger_validate"
        ;;
      validate)
        echo "DISPATCH:session_ready"
        ;;
      *)
        echo "DISPATCH:noop phase=$PHASE"
        ;;
    esac
    ;;
  session_complete)
    echo "DISPATCH:finalize session=$SESSION_NAME"
    ;;
  validation_failed)
    echo "DISPATCH:report_errors session=$SESSION_NAME"
    ;;
  model_selected)
    echo "DISPATCH:trigger_generate session=$SESSION_NAME"
    ;;
  *)
    echo "DISPATCH:unknown event=$EVENT"
    ;;
esac
