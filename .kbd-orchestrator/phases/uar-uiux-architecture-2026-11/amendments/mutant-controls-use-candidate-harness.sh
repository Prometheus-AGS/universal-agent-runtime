#!/usr/bin/env bash
# Boundary command 23 (scn-sys-001) was BLOCKED: the BDD harness opened the app through the Vite
# dev proxy, which sends frontend/vendor/a2ui-web-core/src/universal/a2ui-lit-element.ts to the
# browser with a decorator untransformed, so the page cannot render a conversation. The baseline
# commit fails the same way. Candidate c8eb9e23 points the harness at the built page the runtime
# serves (tests/bdd/playwright.config.ts only). The two mutant controls, neg-sys-015 (M2) and
# neg-sys-017 (M1), run inside their own worktrees, created from the previous candidate, and so
# would still use the old harness config. This amendment makes each of them copy the candidate's
# tests/bdd/playwright.config.ts into its worktree before it runs. The mutant binaries, built
# pages and patches are untouched, so the mutant builds are not redone. No feature file, oracle,
# control expectation, budget, verdict or BIND change. Signs, re-adopts, checks readiness.
# Run from the repository root.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-21-mutant-controls-use-candidate-harness.json"
PREV="$(jq -er .identity.contractDigest "$P/adopted-contract.json" | cut -c1-8)"
uv run --no-project python - <<'PY'
import json,re
p='.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/adopted-contract.json'
c=json.load(open(p))
pat=re.compile(r'\(cd "\$HOME/\.claude/worktrees/(s1-mutant-m[12])" && \{')
rep=r'cp tests/bdd/playwright.config.ts "$HOME/.claude/worktrees/\1/tests/bdd/playwright.config.ts" && (cd "$HOME/.claude/worktrees/\1" && {'
n=0; ids=[]
for cmd in c['boundaryPacket']['commands']:
    out=[]
    for a in cmd['argv']:
        a,k=pat.subn(rep,a)
        if k: ids.append(cmd['id'])
        n+=k; out.append(a)
    cmd['argv']=out
assert n==2 and sorted(ids)==['neg-sys-015','neg-sys-017'], (n,ids)
c['amendmentHistory'].append({'date':'2026-11-21','supersedesDigest':c['identity']['contractDigest'],
 'reason':'scn-sys-001 was BLOCKED: the Vite dev proxy the BDD harness used cannot render a conversation (a vendored A2UI file reaches the browser with a decorator untransformed; the baseline commit fails the same way). Candidate c8eb9e23 points the harness at the built page the runtime serves. The mutant controls run in worktrees created from the previous candidate and would still use the old harness config.',
 'changes':['boundaryPacket.commands neg-sys-015 and neg-sys-017: each first copies the candidate tests/bdd/playwright.config.ts into its mutant worktree'],
 'unchanged':'all spec text, feature files, scenarioMatrix, oracles, control expectations, verdict rules, budgets, BIND checks, mutant patches and builds, and command order',
 'operatorApproval':'signed by the owner'})
json.dump(c,open(p,'w'),indent=2,ensure_ascii=False); open(p,'a').write('\n')
print('updated',n,'controls:',ids)
PY
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "Mutant controls copy the fixed BDD harness config into their worktrees before running. No scenario or test change." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes "contract-adoption-uar-uiux-architecture-2026-11-$PREV" --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
