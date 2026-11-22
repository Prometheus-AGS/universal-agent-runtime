"""Edit the working S1 contract for repair round 4 (operator "Grant as proposed", 2026-11-22).

Usage: repair-round-4.py <contract.json>. Every edit asserts the value it replaces.
Run only through repair-round-4.sh, after the round 4 commit exists and the operator approved signing.
"""
import hashlib
import json
import sys

P = '.kbd-orchestrator/phases/uar-uiux-architecture-2026-11'
FEATURE = 'tests/bdd/features/thread-menu.feature'
p = sys.argv[1]
c = json.load(open(p))
assert c['identity']['contractDigest'].startswith('215f15bd'), c['identity']['contractDigest']
feature_sha = hashlib.sha256(open(FEATURE, 'rb').read()).hexdigest()

b = c['budgets']
assert (b['repairAttempts'], b['timeMinutes']) == (3, 900), b
b['repairAttempts'] = 4
b['timeMinutes'] = 1350

ids = [s['id'] for s in c['scenarioMatrix']]
assert 'SCN-THR-001' not in ids and len(ids) == 13, ids
template = next(s for s in c['scenarioMatrix'] if s['id'] == 'SCN-SYS-007')
keys = list(template['invalidationKeys']) + [
    'frontend/src/components/layout/left-sidebar.tsx', FEATURE, 'tests/bdd/steps/thread-menu.steps.ts']
SCOPE = ('S1 round 4 (operator grant 2026-11-22): the thread row menu renames and deletes a conversation; '
         'it was dead since the Radix to Base UI swap (e92670e2, onSelect where Base UI needs onClick)')


def scenario(sid, name, outcome, entry, observation, ctl_title, rejection, cmd, limits):
    return {
        'id': sid, 'name': name, 'actor': 'person chatting', 'usefulOutcome': outcome,
        'sourceRequirement': 'review/repair-round-4-proposal.md section 2',
        'reasonNow': 'boundary attempt 4: thread Rename is dead (SCN-SYS-011..015 FAIL); Delete is permanent and untested',
        'likelihood': 'HIGH', 'mandatory': True, 'scopeRefs': [SCOPE],
        'ownerTask': 'repair-round-4/T1', 'productionEntry': entry,
        'realCollaborators': template['realCollaborators'], 'substitutedBoundaries': ['model vendor (stub-llm)'],
        'oracle': {'observation': observation,
                   'expectedTruthSource': f'`{FEATURE}` (sha256 {feature_sha}) {sid}',
                   'comparator': 'exact visible text match in the sidebar thread list, before and after reload'},
        'negativeControl': {
            'input': (f'the same scenario on the baseline worktree f2c9e36 (menu items use onSelect, so they do '
                      f'nothing); the cancel control `{ctl_title}` runs inside the scenario command'),
            'expectedRejection': rejection, 'commandRef': cmd},
        'fixtures': (f'scratch data root and ephemeral ports; stub fixture tests/bdd/fixtures/bdd-chat.json '
                     f'("What is 2 plus 2?"). Frozen file: {FEATURE} (sha256 {feature_sha})'),
        'evidence': 'Playwright JSON result, trace and video per sample; baseline JSON result for the control',
        'limitations': limits, 'invalidationKeys': keys,
        'restoration': 'stop the services and delete the scratch root',
        'nondeterministic': True, 'stochastic': template['stochastic']}


c['scenarioMatrix'] += [
    scenario('SCN-THR-001', 'Renaming a thread from its row menu keeps the new name after a reload',
             'a person can rename a conversation and the name survives a reload',
             'sidebar row menu "Actions for <title>" > Rename > dialog "Rename thread" > Save',
             'sidebar thread list text after Save and after a page reload; active row title',
             'Cancelling the rename dialog keeps the thread\'s title',
             'the scenario fails on the baseline (the Rename dialog never opens)', 'neg-thr-001',
             ['embedded store only: P2 (remote snapshot titles every thread "New conversation") is not reachable here',
              'old-title absence is checked on the active row only, because synced rows may share the placeholder']),
    scenario('SCN-THR-002', 'Deleting a thread from its row menu removes it for good and keeps the others',
             'a person can delete one conversation; it stays gone after a reload and the others remain',
             'sidebar row menu "Actions for <title>" > Delete > alertdialog > Delete',
             'confirmation text; sidebar thread list after confirm and after a page reload',
             'Cancelling the delete confirmation keeps the thread',
             'the scenario fails on the baseline', 'neg-thr-002',
             ['threads A and B are named through Rename in setup, so on the baseline the control fails at that setup '
              'step, not at Delete: it proves the test cannot pass on a dead menu, not that the Delete step alone '
              'would catch a dead Delete',
              'embedded store only: it sends no thread snapshot, so D1 (a postgres or remote-Surreal snapshot '
              're-adding a deleted thread, untitled) cannot show here; absence is checked by title',
              'the dialog closes only after the local database write resolves; the reload proves the browser '
              'database, not the server session'])]

cmds = c['boundaryPacket']['commands']
assert len(cmds) == 50 and cmds[49]['id'] == 'rebind-postcondition', [x['id'] for x in cmds[-2:]]
ref = next(x for x in cmds if x['id'] == 'scn-sys-007')['argv'][2]
gate, run = ref.split(' && { ', 1)
assert run.startswith('env pnpm exec'), run[:40]
gate = gate.replace('| shasum -a 256 -c --status',
                    f'"{feature_sha}  {FEATURE}" | shasum -a 256 -c --status', 1)
assert FEATURE in gate
pw = ('pnpm exec env -u UAR_STATIC_DIR CI=1 UAR_SERVER__GRPC_PORT=50061 '
      'UAR_SERVER__INGEST_DIRECTORY="$(mktemp -d)" playwright test -c tests/bdd/playwright.config.ts')


def scn_cmd(sid, titles):
    grep = '|'.join(titles)
    return {'id': sid.lower().replace('scn-', 'scn-'), 'kind': 'scenario', 'scenarioIds': [sid], 'cwd': '.',
            'argv': ['bash', '-c', f"{gate} && {{ env {pw} --grep '{grep}' --repeat-each 3 --retries 0; }}"]}


def neg_cmd(sid, title, out):
    body = (f'mkdir -p "{P}/boundary" && R="$PWD" && rm -f "{P}/boundary/{out}" && '
            f'rsync -a --delete --exclude fixtures/live-model-choice-new-kinds.json tests/bdd/ '
            f'"$HOME/.claude/worktrees/uiux-baseline/tests/bdd/" && cd "$HOME/.claude/worktrees/uiux-baseline" && '
            f'pnpm exec bddgen test -c tests/bdd/playwright.config.ts && '
            f'{{ env -u STUB_LLM_EXTRA_FIXTURES {pw} --retries 0 --grep \'{title}\' --reporter=json '
            f'> "$R/{P}/boundary/{out}"; true; }} && cd "$R" && '
            f'jq -e --arg t \'{title}\' \'[.. | objects | select(has("specs")) | .specs[] | select(.title == $t)] '
            f'| length == 1 and (.[0].ok == false)\' "{P}/boundary/{out}"')
    return {'id': 'neg-' + sid.lower()[4:], 'kind': 'negativeControl', 'scenarioIds': [sid], 'cwd': '.',
            'argv': ['bash', '-c', f'{gate} && {{ {body}; }}']}


T1 = 'Renaming a thread from its row menu keeps the new name after a reload'
C1 = "Cancelling the rename dialog keeps the thread's title"
T2 = 'Deleting a thread from its row menu removes it for good and keeps the others'
C2 = 'Cancelling the delete confirmation keeps the thread'
new = [scn_cmd('SCN-THR-001', [T1, C1.replace("'", '.')]), neg_cmd('SCN-THR-001', T1, 'baseline-thr-001.json'),
       scn_cmd('SCN-THR-002', [T2, C2]), neg_cmd('SCN-THR-002', T2, 'baseline-thr-002.json')]
c['boundaryPacket']['commands'] = cmds[:49] + new + cmds[49:]

ee = c['boundaryPacket']['expectedEffects']
ee.append(
    'Repair round 4 adds SCN-THR-001 and SCN-THR-002 (tests/bdd/features/thread-menu.feature, frozen by digest in '
    'commands 49-52 only). Each scenario command runs the scenario and its cancel control, 3 samples each, '
    '--retries 0. Each negative control overlays the candidate tests on the baseline worktree f2c9e36, whose menu '
    'items use onSelect, and requires the scenario to fail there (JSON result ok == false). Commands 2-6 carry '
    'forward (baseline unchanged). Run order for this candidate: 0, 1, 7, 8, 23, 49-52, 9-22, 24-48, 53. '
    'rebind-postcondition moves from index 49 to 53, unchanged.')
c['amendmentHistory'].append({
    'date': '2026-11-22', 'supersedesDigest': c['identity']['contractDigest'],
    'reason': ('The operator granted repair round 4 as proposed (review/repair-round-4-proposal.md): T1 thread '
               'menu, P1 send race, harness fixes C1, H1, H2, H3, and tests for Rename and Delete.'),
    'changes': ['budgets.repairAttempts 3 to 4', 'budgets.timeMinutes 900 to 1350, cumulative',
                'scenarioMatrix adds SCN-THR-001 and SCN-THR-002 (13 to 15)',
                'commands 49-52 added (scn-thr-001, neg-thr-001, scn-thr-002, neg-thr-002); rebind to index 53',
                'expectedEffects: run order and the THR controls'],
    'unchanged': ('every existing scenario, oracle, control, verdict rule, command text, BIND check and frozen '
                  'fixture; all attempt-4 verdicts are void by the Repair rule; consumed budgets and history are '
                  'not reset'),
    'operatorApproval': 'signed by the owner'})
json.dump(c, open(p, 'w'), indent=2, ensure_ascii=False)
open(p, 'a').write('\n')
print('contract edited:', p, 'feature', feature_sha[:12])
