#!/usr/bin/env bash
# The independent critic BLOCKED the inventory for candidate c8eb9e23 (review/s1/inventory-confirmation.json):
#   HIGH   the plan kept the M1/M2 worktrees built from 2723a97b, but the contract's Repair rule
#          (boundaryPacket.expectedEffects[28]) requires them to be removed, recreated and rebuilt.
#   MEDIUM three contract statements still describe the Vite dev-server harness; static/ is now the
#          page under test for BDD commands but nothing pins which static/ the runtime serves.
#   MEDIUM the binding check never reads the critic's verdict.
# This amendment makes the contract say what the harness now does. It always:
#   1. rewrites releaseClaim.explicitUnknowns[7], the static/ clause of expectedEffects[29] and cleanup[0];
#   2. runs every BDD command (tests/bdd/playwright.config.ts) with UAR_STATIC_DIR unset, so the
#      runtime serves ./static of the checkout it runs in;
#   3. adds `verdict == PASS` on the critic's record to the binding check of all 50 commands.
# With --keep-mutant-builds it also records a one-time exception to the Repair rule for c8eb9e23 and
# makes neg-sys-015 / neg-sys-017 check, after copying the harness config, that the mutant worktree
# differs from the candidate commit only in its target file (static/ excluded). Without the flag the
# Repair rule stands as written and both mutants are rebuilt.
# No feature file, oracle, control expectation, budget or verdict rule changes.
# Signs, re-adopts, checks readiness. Run from the repository root.
set -euo pipefail
KEEP=0; [ "${1:-}" = "--keep-mutant-builds" ] && KEEP=1
P=.kbd-orchestrator/phases/uar-uiux-architecture-2026-11
KP="$HOME/.prometheus/plugins/prometheus-skill-pack/current"
M="$P/amendments/2026-11-21-built-page-harness-terms.json"
PREV="$(jq -er .identity.contractDigest "$P/adopted-contract.json" | cut -c1-8)"
KEEP=$KEEP uv run --no-project python - <<'PY'
import json,os,subprocess
keep=os.environ['KEEP']=='1'
P='.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/'
p=P+'adopted-contract.json'
c=json.load(open(p))
bp=c['boundaryPacket']; cmds=bp['commands']
CONF=P+'review/s1/inventory-confirmation.json'
INV=P+'implementation-complete-inventory.json'

# 1. statements
u=c['releaseClaim']['explicitUnknowns']
assert 'dev server (bun run dev on 8085)' in u[7], u[7]
u[7]=('The BDD path opens the built frontend that the runtime serves from static/ (app server, port 3002) and runs '
      'target/debug binaries. BDD commands run with UAR_STATIC_DIR unset, so the runtime serves ./static of the checkout '
      'it runs in. static/ is outside the binding check; prepare-bdd rebuilds it after the bind. The non-S1 feature '
      'product-screen-validation waits for a dev-build hook (window.__uarRuntimeReplay) and cannot pass on the built page; '
      'no boundary command selects it. The Vite dev server cannot render a conversation (a vendored A2UI file reaches the '
      'browser with a decorator untransformed); that fault predates S1 and is not fixed here. UAR_STATIC_DIR is set only by '
      'the supporting real-server runs and the baseline supporting check')
old='and is served only to supporting real-server starts through UAR_STATIC_DIR'
assert old in bp['expectedEffects'][29]
bp['expectedEffects'][29]=bp['expectedEffects'][29].replace(old,
  'is the page every BDD command opens (served by the runtime from ./static with UAR_STATIC_DIR unset), and is the page '
  'supporting real-server starts serve through UAR_STATIC_DIR')
old='which only the independent critic writes.'
assert old in bp['expectedEffects'][29]
bp['expectedEffects'][29]=bp['expectedEffects'][29].replace(old, old+' The binding check also requires that record\'s verdict to be PASS.')
assert 'frontend dev server' in bp['cleanup'][0], bp['cleanup'][0]
bp['cleanup'][0]='stop stub-llm and the UAR server started by Playwright'

# 2. UAR_STATIC_DIR unset for BDD commands
s='pnpm exec env CI=1 UAR_SERVER__GRPC_PORT'; n_env=0
for x in cmds:
    a=x['argv'][2]
    if 'tests/bdd/playwright.config.ts' in a and s in a:
        assert a.count(s)==1, x['id']
        x['argv'][2]=a.replace(s,'pnpm exec env -u UAR_STATIC_DIR CI=1 UAR_SERVER__GRPC_PORT'); n_env+=1
assert n_env==21, n_env

# 3. verdict in BIND
t='test "$(jq -er .candidateCommit %s)" = "$C"' % CONF; n_bind=0
for x in cmds:
    a=x['argv'][2]; assert a.count(t)==1, x['id']
    x['argv'][2]=a.replace(t, t+' && test "$(jq -er .verdict %s)" = "PASS"' % CONF); n_bind+=1
assert n_bind==50, n_bind

changes=['releaseClaim.explicitUnknowns[7], boundaryPacket.expectedEffects[29] and boundaryPacket.cleanup[0] describe the built-page BDD harness',
         '21 BDD commands (baseline-check-bdd and commands 23-42) run Playwright with UAR_STATIC_DIR unset',
         'the binding check of all 50 commands also requires verdict PASS in review/s1/inventory-confirmation.json']
unchanged='all spec text, feature files, scenarioMatrix, oracles, control expectations (the jq judgements), verdict rules, budgets, frozen digests, mutant patches and command order'

# 4. optional exception
if keep:
    ex=(' Round-2 exception, signed by the owner on 2026-11-21 and valid for candidate c8eb9e23 only: that commit differs '
        'from 2723a97b in tests/bdd/playwright.config.ts alone, so the M1 and M2 worktrees created, digest-checked, patched '
        'and built from 2723a97b are kept and commands 9-22 are not rerun. neg-sys-015 and neg-sys-017 copy the candidate '
        'tests/bdd/playwright.config.ts into their worktree and then require that the worktree differs from the candidate '
        'commit only in the inventory target file (static/ excluded); if not, the control is INVALID. bind-candidate-tree, '
        'prepare-bdd and both baseline checks rerun as the rule says.')
    assert bp['expectedEffects'][28].startswith('Repair rule:')
    bp['expectedEffects'][28]+=ex
    n_mut=0
    for wt,mk in (('s1-mutant-m2','M2'),('s1-mutant-m1','M1')):
        cp='cp tests/bdd/playwright.config.ts "$HOME/.claude/worktrees/%s/tests/bdd/playwright.config.ts" && ' % wt
        chk=('test "$(git -C "$HOME/.claude/worktrees/%s" diff --name-only "$C" -- . \':(exclude)static\')" = '
             '"$(jq -er .mutants.%s.targetFile %s)" && ') % (wt,mk,INV)
        for x in cmds:
            a=x['argv'][2]
            if cp in a:
                assert a.count(cp)==1 and x['id'] in ('neg-sys-015','neg-sys-017'), x['id']
                x['argv'][2]=a.replace(cp,cp+chk); n_mut+=1
    assert n_mut==2, n_mut
    changes+=['boundaryPacket.expectedEffects[28] (Repair rule): one-time exception for c8eb9e23 keeping the M1/M2 worktrees and builds made from 2723a97b',
              'neg-sys-015 and neg-sys-017: after the harness copy, require the mutant worktree to differ from the candidate commit only in its target file (static/ excluded)']
    unchanged+='. Mutant binaries and built pages are not rebuilt (exception above)'
else:
    unchanged+='. The Repair rule stands as written: both mutants are removed, recreated and rebuilt from c8eb9e23'

# every command must still be valid shell (syntax check only; nothing is executed)
for x in cmds:
    r=subprocess.run(['bash','-n','-c',x['argv'][2]],capture_output=True,text=True)
    assert r.returncode==0, (x['id'],r.stderr[:300])

c['amendmentHistory'].append({'date':'2026-11-21','supersedesDigest':c['identity']['contractDigest'],
 'reason':'Independent critic BLOCK on the inventory for c8eb9e23: the contract still described the Vite dev-server BDD harness after the harness moved to the built page; static/ became the page under test with nothing pinning which static/ is served; the binding check ignored the critic\'s verdict; and the plan to keep the mutant builds contradicted the Repair rule without a recorded owner decision.',
 'changes':changes,'unchanged':unchanged,'operatorApproval':'signed by the owner'})
json.dump(c,open(p,'w'),indent=2,ensure_ascii=False); open(p,'a').write('\n')
print('keep-mutant-builds' if keep else 'rebuild-mutants','| env',n_env,'| bind',n_bind)
PY
if [ "$KEEP" = 1 ]; then R="Contract describes the built-page BDD harness; binding requires the critic's PASS; one-time exception keeps the mutant builds for c8eb9e23. No scenario or test change."
else R="Contract describes the built-page BDD harness; binding requires the critic's PASS; mutants are rebuilt per the Repair rule. No scenario or test change."; fi
bash "$KP/shared/scripts/kbd-scope-authority-template.sh" amend-contract --phase uar-uiux-architecture-2026-11 \
  --contract "$P/adopted-contract.json" --reason "$R" > "$M"
ssh-keygen -Y sign -f ~/.ssh/id_ed25519 -n prometheus-kbd-scope-amendment "$M"
bash "$KP/shared/scripts/kbd-contract-adopt.sh" "$P/adopted-contract.json" \
  --supersedes "contract-adoption-uar-uiux-architecture-2026-11-$PREV" --amendment "$M" --signature "$M.sig"
bash "$KP/shared/scripts/kbd-phase-readiness-check.sh" --phase uar-uiux-architecture-2026-11
