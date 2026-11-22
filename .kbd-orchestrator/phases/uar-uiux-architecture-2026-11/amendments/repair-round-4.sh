#!/usr/bin/env bash
# Repair round 4, granted by the operator on 2026-11-22 ("Grant as proposed", review/repair-round-4-proposal.md).
# Changes: repairAttempts 3 -> 4; timeMinutes 900 -> 1350; adds SCN-THR-001 and SCN-THR-002 with their
# scenario and baseline negative-control commands (49-52; rebind moves to 53); run-order note. Nothing else.
# Run from the repository root, at the round 4 commit, and only after the operator approves signing.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-22-repair-round-4.json"
PREV="$(jq -er .identity.contractDigest "$P/adopted-contract.json" | cut -c1-8)"
test "$PREV" = 215f15bd
uv run --no-project python "$P/amendments/repair-round-4.py" "$P/adopted-contract.json"
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "Repair round 4: fourth repair round, its rerun time, and tests for thread Rename and Delete (SCN-THR-001, SCN-THR-002)." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes "contract-adoption-uar-uiux-architecture-2026-11-$PREV" --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
