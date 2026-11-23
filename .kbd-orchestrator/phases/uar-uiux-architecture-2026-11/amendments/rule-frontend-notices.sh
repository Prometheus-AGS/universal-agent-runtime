#!/usr/bin/env bash
# Operator ruling 2026-11-21 ("Rule them harmless"): exempt two pre-existing third-party
# frontend bundler notices from the clean-build stop condition, sign the amend-contract
# manifest with the owner's key, re-adopt, and rerun readiness. No scenario, oracle,
# control, budget or command change. Run from the repository root.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-21-third-party-frontend-notices.json"
uv run --no-project python - <<'PY'
import json
p='.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/adopted-contract.json'
c=json.load(open(p))
sc=c['boundaryPacket']['stopConditions']
anchor='nothing in it can be changed here)'
i=next(k for k,s in enumerate(sc) if anchor in s)
assert 'Some chunks are larger' not in sc[i]
add=(', and two pre-existing third-party frontend bundler notices (operator ruling 2026-11-21, decision-log.md;'
     ' S1 changed no frontend dependency, lockfile or bundler configuration):'
     ' the Vite/rolldown advisory `(!) Some chunks are larger than 1100 kB after minification` provided every chunk over'
     ' 1100 kB is `elk-*.js` (the elk layout engine bundled by mermaid), and `[EVAL] Use of direct `eval`` notices whose'
     ' reported location is inside `node_modules/.pnpm/@electric-sql+pglite@`; any other chunk over the limit or any'
     ' other eval location is not exempt')
sc[i]=sc[i].replace(anchor, anchor+add, 1)
c['amendmentHistory'].append({'date':'2026-11-21','supersedesDigest':c['identity']['contractDigest'],
 'reason':'Boundary command 1 (prepare-bdd) built successfully but stopped on two Vite/rolldown notices from third-party code: the elk-XV6XMXQE chunk (mermaid elk layout engine, 1498.93 kB) exceeds chunkSizeWarningLimit 1100, and pglite 0.5.12 dist uses direct eval. Both predate S1 (no change to frontend/package.json, pnpm-lock.yaml or vite.config.ts in f2c9e36..ae01b21c).',
 'changes':['boundaryPacket.stopConditions clean-build rule: exempt the elk-only chunk-size advisory and pglite direct-eval notices'],
 'unchanged':'all spec text, scenarioMatrix, oracles, controls, verdict rules, budgets and commands',
 'operatorApproval':'operator ruling "Rule them harmless", signed by the owner'})
json.dump(c,open(p,'w'),indent=2,ensure_ascii=False); open(p,'a').write('\n')
PY
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "Exempt two pre-existing third-party frontend bundler notices (elk-only chunk size, pglite direct eval) from the clean-build stop condition. No scenario or command change." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes contract-adoption-uar-uiux-architecture-2026-11-32717a4e --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
