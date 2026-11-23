#!/usr/bin/env bash
# Close uar-uiux-architecture-2026-11 under the superseding exit (operator, 2026-11-23).
# 1. Backs up the KBD registry and re-registers this checkout, so the lock probe runs again
#    outside a sandbox and clears the read-only classification saved at 2026-11-23T10:10:15Z.
# 2. Signs (ssh-keygen asks to overwrite the existing .sig; answer y) and adopts the contract.
# 3. Writes boundary-results.json from the recorded rc.6 BDD run and runs the close evaluator.
# Run from the repository root in a normal terminal, on the owner's instruction only.
set -euo pipefail
cd /Volumes/T7/Projects/prometheus/universal-agent-runtime
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
D="$HOME/Library/Application Support/prometheus/kbd"
REPLICA=1cf1630a-e1a0-439e-b801-9f948eead3bf

cp -p "$D/registry.json" "$D/registry-maintenance-backups/registry.json.bak.$(date -u +%Y%m%dT%H%M%SZ)-before-reregister-$REPLICA"
prometheus kbd register .
STATE="$(jq -r --arg r "$REPLICA" '.. | objects | select(.replicaId? == $r) | .readOnly' "$D/registry.json")"
echo "replica $REPLICA readOnly=$STATE"
[ "$STATE" = "false" ] || { echo "still read-only; stopping before signing" >&2; exit 1; }

bash "$P/amendments/superseding-exit.sh"

DISP="$(prometheus kbd --path . phase disposition --id uar-uiux-architecture-2026-11 --json)"
BOUND="$(printf '%s' "$DISP" | jq -r '.binding.contractDigest')"
[ "$BOUND" != "a6b514f2c0b0cbd2d6f6d14d6ff58588f13fce5a957cc22f928ce916c240d0f9" ] || { echo "contract not re-bound; stopping" >&2; exit 1; }
uv run --no-project python "$P/amendments/superseding-exit-evidence.py" "$BOUND" \
  "$(printf '%s' "$DISP" | jq -r '.projectId')" "$(printf '%s' "$DISP" | jq -r '.runId')"
bash "$KP/shared/scripts/kbd-phase-close-eval.sh" --phase uar-uiux-architecture-2026-11
