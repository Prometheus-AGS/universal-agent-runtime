#!/usr/bin/env bash
# Boundary command 14 (install-mutant-m1-deps) installed the root and frontend workspaces, then
# failed on the third install: frontend/packages/prometheus-entity-management pins pnpm 10.33.0
# (packageManager and devEngines.packageManager, onFail "error"), and pnpm 11.15.0 run through
# corepack refuses ("This project is configured to use 10.33.0 of pnpm"). The same install is in
# install-mutant-m2-deps. pnpm 10.33.0 is in the local corepack cache; with network disabled,
# `corepack pnpm@10.33.0 --dir <PEM> install --frozen-lockfile` completed in 22 s in the M1
# worktree. This amendment runs exactly that for the two PEM installs. No code, feature file,
# oracle, control, budget, verdict or BIND change. Signs, re-adopts, checks readiness.
# Run from the repository root.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-21-pem-install-pinned-pnpm.json"
PREV="$(jq -er .identity.contractDigest "$P/adopted-contract.json" | cut -c1-8)"
uv run --no-project python - <<'PY'
import json,re
p='.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/adopted-contract.json'
c=json.load(open(p))
pat=re.compile(r'pnpm --dir ("\$HOME/\.claude/worktrees/s1-mutant-m[12]/frontend/packages/prometheus-entity-management") install --frozen-lockfile')
n=0
for cmd in c['boundaryPacket']['commands']:
    out=[]
    for a in cmd['argv']:
        a,k=pat.subn(r'COREPACK_ENABLE_NETWORK=0 corepack pnpm@10.33.0 --dir \1 install --frozen-lockfile',a)
        n+=k; out.append(a)
    cmd['argv']=out
assert n==2, n
c['amendmentHistory'].append({'date':'2026-11-21','supersedesDigest':c['identity']['contractDigest'],
 'reason':'install-mutant-m1-deps failed on the prometheus-entity-management install: the package pins pnpm 10.33.0 (devEngines onFail error) and pnpm 11.15.0 under corepack refuses. install-mutant-m2-deps has the same install. pnpm 10.33.0 is cached locally and installs the M1 PEM lockfile offline in 22 s.',
 'changes':['boundaryPacket.commands install-mutant-m1-deps and install-mutant-m2-deps: the PEM install runs as `COREPACK_ENABLE_NETWORK=0 corepack pnpm@10.33.0 --dir <PEM> install --frozen-lockfile`'],
 'unchanged':'all spec text, feature files, scenarioMatrix, oracles, controls, verdict rules, budgets, BIND checks, the root and frontend installs, and command order',
 'operatorApproval':'signed by the owner'})
json.dump(c,open(p,'w'),indent=2,ensure_ascii=False); open(p,'a').write('\n')
print('updated',n,'installs')
PY
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "Install the entity-management package with the pnpm version it pins (10.33.0, cached locally). No scenario or test change." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes "contract-adoption-uar-uiux-architecture-2026-11-$PREV" --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
