#!/usr/bin/env bash
# Restamp method.specDeltaDigest in the S1 contract to the current OpenSpec deltas,
# sign the amend-contract manifest with the owner's key, re-adopt, and rerun readiness.
# No spec, scenario, oracle or command changes. Run from the repository root.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-21-spec-digest-restamp.json"
NEW="$(bash "$KP/shared/scripts/kbd-spec-delta-digest.sh" --phase uar-uiux-architecture-2026-11)"
python3 - "$NEW" <<'PY'
import json,sys
p='.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/adopted-contract.json'
c=json.load(open(p)); old=c['method']['specDeltaDigest']; c['method']['specDeltaDigest']=sys.argv[1]
c['amendmentHistory'].append({'date':'2026-11-21','supersedesDigest':c['identity']['contractDigest'],
 'reason':'Readiness check REJECT-STALE: method.specDeltaDigest ('+old[:8]+') was stamped before the final spec edits (all delta files last modified before the 2026-11-20 13:35 adoption) and never restamped.',
 'changes':['method.specDeltaDigest restamped to the current deltas of the four registered changes'],
 'unchanged':'all spec text, scenarioMatrix, oracles, controls, verdict rules, budgets and commands',
 'operatorApproval':'signed by the owner'})
json.dump(c,open(p,'w'),indent=2,ensure_ascii=False); open(p,'a').write('\n')
PY
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "Restamp method.specDeltaDigest to the spec deltas as adopted. No spec, scenario or command change." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes contract-adoption-uar-uiux-architecture-2026-11-3f2505fe --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
