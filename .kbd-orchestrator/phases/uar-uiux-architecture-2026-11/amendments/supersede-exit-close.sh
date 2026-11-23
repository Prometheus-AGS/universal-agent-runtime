#!/usr/bin/env bash
# Close uar-uiux-architecture-2026-11 as CLOSED_SUPERSEDED (operator, 2026-11-23: "Yes, close it").
# The manifest was emitted by kbd-scope-authority-template.sh supersede-exit and passed
# `prometheus kbd phase validate-exit` before signing. This signs it with the owner's key
# and records the disposition. Run from the repository root, on the owner's instruction only.
set -euo pipefail
cd /Volumes/T7/Projects/prometheus/universal-agent-runtime
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-23-supersede-exit.json"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-phase-close-eval.sh" --phase uar-uiux-architecture-2026-11 \
  --superseded "$M" --signature "$M.sig"
prometheus kbd --path . phase disposition --id uar-uiux-architecture-2026-11 --json | jq '{disposition, closure}'
