#!/usr/bin/env bash
# The operator approved a third B5 repair round on 2026-11-22 (decision-log.md, repair-round-3.md).
# The adopted contract allows two, and its 180-minute time budget is shorter than one rebuild and
# rerun of the boundary. This amendment changes two budget values and nothing else:
#   budgets.repairAttempts  2   -> 3
#   budgets.timeMinutes     180 -> 600
# Consumed budgets and history are not reset. No scenario, oracle, control, verdict rule, command,
# frozen fixture or binding check changes. The Repair rule applies in full: the round 2 exception
# that kept the mutant builds was for c8eb9e23 only.
# Signs, re-adopts, checks readiness. Run from the repository root.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-22-repair-round-3-budget.json"
PREV="$(jq -er .identity.contractDigest "$P/adopted-contract.json" | cut -c1-8)"
uv run --no-project python - <<'PY'
import json
p='.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/adopted-contract.json'
c=json.load(open(p))
b=c['budgets']
assert (b['repairAttempts'],b['timeMinutes'])==(2,180), b
b['repairAttempts']=3; b['timeMinutes']=600
c['amendmentHistory'].append({'date':'2026-11-22','supersedesDigest':c['identity']['contractDigest'],
 'reason':'The operator approved a third B5 repair round after the c8eb9e23 boundary (1 PASS, 2 FAIL, 10 BLOCKED): models-page DOM size, PGlite seed migration, BDD local principal, rate-limit burst or test configuration. See repair-round-3.md.',
 'changes':['budgets.repairAttempts 2 to 3','budgets.timeMinutes 180 to 600'],
 'unchanged':'all spec text, feature files, scenarioMatrix, oracles, controls, verdict rules, commands, BIND checks, frozen fixtures and the Repair rule; consumed budgets and history are not reset',
 'operatorApproval':'signed by the owner'})
json.dump(c,open(p,'w'),indent=2,ensure_ascii=False); open(p,'a').write('\n')
print('budgets updated')
PY
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "Allow the operator-approved third repair round and its rerun time. No scenario or test change." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes "contract-adoption-uar-uiux-architecture-2026-11-$PREV" --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
