#!/usr/bin/env bash
# Repair round 4b: re-freeze thread-menu.feature ("Reply with OK."), after the inventory critic found the
# baseline stub cannot answer "What is 2 plus 2?". Only that file's digest and the THR fixtures text change.
#
# Run from the repository root, at the round 4 commit, and only after the operator approves signing.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-22-repair-round-4b.json"
PREV="$(jq -er .identity.contractDigest "$P/adopted-contract.json" | cut -c1-8)"
test "$PREV" = d5bb7921
uv run --no-project python "$P/amendments/repair-round-4b.py" "$P/adopted-contract.json"
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "Repair round 4b: thread-menu.feature sends a message the baseline stub answers, so the THR negative controls reach the menu." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes "contract-adoption-uar-uiux-architecture-2026-11-$PREV" --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
