#!/usr/bin/env bash
# Combined rc.6 candidate (operator, 2026-11-22: "Yes, combine them"). Five changes, all in rc6-candidate.py:
#   1. the ruled frontend-notice clause admits main's dependency updates;
#   2. the owner accepts that SCN-SYS-012 does not show an oracle rejecting the old build;
#   3. one chat scenario runs before the mutant builds;
#   4. an S1 pass certifies neither main, #539 nor the images;
#   5. budgets.timeMinutes 600 -> 900 (cumulative; nothing consumed is reset).
# No scenario, oracle, control, verdict rule, command, frozen fixture or repair budget changes.
# Signs, re-adopts, checks readiness. Run from the repository root, on the owner's instruction only.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-22-rc6-candidate.json"
PREV="$(jq -er .identity.contractDigest "$P/adopted-contract.json" | cut -c1-8)"
uv run --no-project python "$P/amendments/rc6-candidate.py" "$P/adopted-contract.json"
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "Combined rc.6 candidate: notice clause, 012 baseline limit, run order, exclusion and time budget. No scenario or test change." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes "contract-adoption-uar-uiux-architecture-2026-11-$PREV" --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
