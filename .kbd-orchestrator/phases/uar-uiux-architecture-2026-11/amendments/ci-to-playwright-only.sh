#!/usr/bin/env bash
# Boundary command 7 (baseline-check-bdd) stopped before any test ran: with CI=1 in its
# environment, pnpm 11 sets enableGlobalVirtualStore, which the existing installs did not
# record, so `pnpm exec` refuses with ERR_PNPM_VERIFY_DEPS_BEFORE_RUN. Every scenario and
# control command has the same prefix. This amendment passes CI=1 to the Playwright process
# only (`pnpm exec env CI=1 playwright ...`), so pnpm's dependency check runs exactly as it
# did at install and Playwright still sees CI=1 (no server reuse, forbidOnly). No scenario,
# feature file, oracle, control, budget or verdict change. Signs, re-adopts, checks readiness.
# Run from the repository root.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-21-ci-to-playwright-only.json"
PREV="$(jq -er .identity.contractDigest "$P/adopted-contract.json" | cut -c1-8)"
uv run --no-project python - <<'PY'
import json
p='.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/adopted-contract.json'
c=json.load(open(p))
pairs=[('CI=1 pnpm exec playwright','pnpm exec env CI=1 playwright'),
       ('env CI=1 pnpm -C frontend exec playwright','pnpm -C frontend exec env CI=1 playwright')]
n=0
for cmd in c['boundaryPacket']['commands']:
    new=[]
    for a in cmd['argv']:
        for old,rep in pairs:
            n+=a.count(old); a=a.replace(old,rep)
        new.append(a)
    cmd['argv']=new
left=sum(' '.join(x['argv']).count('CI=1 pnpm') for x in c['boundaryPacket']['commands'])
assert n>0 and left==0, (n,left)
c['amendmentHistory'].append({'date':'2026-11-21','supersedesDigest':c['identity']['contractDigest'],
 'reason':'Boundary command baseline-check-bdd exited 5 before any test ran: CI=1 in the pnpm environment makes pnpm 11 set enableGlobalVirtualStore, absent from the recorded install state, so pnpm exec refused (ERR_PNPM_VERIFY_DEPS_BEFORE_RUN). Reproduced in both the candidate and baseline checkouts; every scenario and control command shares the prefix.',
 'changes':[f'boundaryPacket.commands: {n} occurrences of `CI=1 pnpm ... exec playwright` rewritten to `pnpm ... exec env CI=1 playwright` (CI reaches Playwright only)'],
 'unchanged':'all spec text, feature files, scenarioMatrix, oracles, controls, verdict rules, budgets, BIND checks and command order',
 'operatorApproval':'signed by the owner'})
json.dump(c,open(p,'w'),indent=2,ensure_ascii=False); open(p,'a').write('\n')
print('rewrote',n,'occurrences')
PY
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "Pass CI=1 to Playwright only so pnpm's dependency check matches the recorded install. No scenario or test change." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes "contract-adoption-uar-uiux-architecture-2026-11-$PREV" --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
