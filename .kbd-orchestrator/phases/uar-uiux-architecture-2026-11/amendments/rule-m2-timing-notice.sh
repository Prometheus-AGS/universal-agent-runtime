#!/usr/bin/env bash
# One-receipt exception for the [PLUGIN_TIMINGS] notice that stopped boundary attempt 4 at command 22.
# Nothing else changes: no runner, command, scenario, oracle, budget or general warning rule.
# Signs, re-adopts, checks readiness. Run from the repository root, on the owner's instruction only.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-22-rule-m2-timing-notice.json"
PREV="$(jq -er .identity.contractDigest "$P/adopted-contract.json" | cut -c1-8)"
uv run --no-project python "$P/amendments/rule-m2-timing-notice.py" "$P/adopted-contract.json"
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "One-receipt exception for a bundler timing notice at build-mutant-m2. No scenario, command or runner change." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes "contract-adoption-uar-uiux-architecture-2026-11-$PREV" --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
