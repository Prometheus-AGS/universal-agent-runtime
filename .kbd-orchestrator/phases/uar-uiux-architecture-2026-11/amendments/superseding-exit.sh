#!/usr/bin/env bash
# Superseding exit (operator, 2026-11-23: "Sign a superseding exit").
# The contract edit is already applied by superseding-exit.py (lint: 15 scenarios clean).
# This writes the manifest, signs it with the owner's key, re-adopts and checks readiness.
# Run from the repository root, on the owner's instruction only.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-23-superseding-exit.json"
PREV=a6b514f2c0b0cbd2d6f6d14d6ff58588f13fce5a957cc22f928ce916c240d0f9
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "Superseding exit: the rc.6 new-code-only BDD check (15 scenarios, run once on 8e4ce275) replaces the unfinished S1 packet. Earlier attempts kept as history; unshipped scope moves to #542." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes "contract-adoption-uar-uiux-architecture-2026-11-${PREV:0:8}" --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11 || true
