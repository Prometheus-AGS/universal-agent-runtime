#!/usr/bin/env bash
# Boundary command 7 (baseline-check-bdd) ran Playwright, but the UAR test server exited 1 before
# any scenario: (1) the live UAR on this Mac (pid 13005, /Users/.../.uar/bin) holds the default
# A2A gRPC port 50051, and the harness does not set UAR_SERVER__GRPC_PORT, so the test server
# fails "Address already in use"; (2) since c065bb07 (2026-10-22) the server creates
# server.ingest_directory (default /data/ingest), which is read-only on macOS, and no harness
# config sets it. Both reproduce from the candidate checkout and the baseline worktree; with both
# settings supplied the server is ready in ~4 s in each. Playwright passes the parent environment
# to its webServers ({...process.env, ...config env}), so this amendment adds
# `UAR_SERVER__GRPC_PORT=50061 UAR_SERVER__INGEST_DIRECTORY="$(mktemp -d)"` to the env that
# already carries CI=1 for Playwright. No code, feature file, oracle, control, budget, verdict or
# BIND change. Signs, re-adopts, checks readiness. Run from the repository root.
set -euo pipefail
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-21-harness-server-settings.json"
PREV="$(jq -er .identity.contractDigest "$P/adopted-contract.json" | cut -c1-8)"
uv run --no-project python - <<'PY'
import json
p='.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/adopted-contract.json'
c=json.load(open(p))
old='exec env CI=1 playwright'
new='exec env CI=1 UAR_SERVER__GRPC_PORT=50061 UAR_SERVER__INGEST_DIRECTORY="$(mktemp -d)" playwright'
n=0
for cmd in c['boundaryPacket']['commands']:
    out=[]
    for a in cmd['argv']:
        n+=a.count(old); out.append(a.replace(old,new))
    cmd['argv']=out
assert n==30, n
c['amendmentHistory'].append({'date':'2026-11-21','supersedesDigest':c['identity']['contractDigest'],
 'reason':'baseline-check-bdd: the UAR test server exited 1 before any scenario. The live UAR on this machine holds the default gRPC port 50051 (no harness sets UAR_SERVER__GRPC_PORT), and since c065bb07 the server must create server.ingest_directory (default /data/ingest, read-only on macOS; no harness sets it). Reproduced from the candidate and baseline checkouts; with both settings the server is ready in ~4 s.',
 'changes':['boundaryPacket.commands: 30 Playwright invocations now also pass UAR_SERVER__GRPC_PORT=50061 and a fresh UAR_SERVER__INGEST_DIRECTORY (mktemp -d) through the environment Playwright hands to its webServers'],
 'unchanged':'all spec text, feature files, scenarioMatrix, oracles, controls, verdict rules, budgets, BIND checks and command order',
 'operatorApproval':'signed by the owner'})
json.dump(c,open(p,'w'),indent=2,ensure_ascii=False); open(p,'a').write('\n')
print('updated',n,'invocations')
PY
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" \
  --reason "Give the test server a free gRPC port and a writable ingest folder. No scenario or test change." > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes "contract-adoption-uar-uiux-architecture-2026-11-$PREV" --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
