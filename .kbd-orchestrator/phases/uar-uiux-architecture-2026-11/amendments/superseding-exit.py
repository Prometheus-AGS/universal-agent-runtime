"""Superseding exit for uar-uiux-architecture-2026-11 (operator, 2026-11-23: "Sign a superseding exit").

Usage: superseding-exit.py <contract.json>. Replaces the S1 boundary packet with the rc.6 exit
check the operator chose on 2026-11-22 ("Limit validation to ONLY new code to exit rc.6 work"):
the 15 BDD scenarios for code new since rc.5, run once with no retries on 8e4ce275.
The four earlier boundary attempts stay on record under boundary/attempt-*; nothing is deleted.
"""
import json
import sys

p = sys.argv[1]
c = json.load(open(p))
prev = c['identity']['contractDigest']
assert prev.startswith('a6b514f2'), prev

PH = '.kbd-orchestrator/phases/uar-uiux-architecture-2026-11'
LMC = 'tests/bdd/features/live-model-choice.feature'
THR = 'tests/bdd/features/thread-menu.feature'
EVID = f'{PH}/rc6-check/3-bdd.json and {PH}/rc6-check/3-bdd-run.log (Playwright result and console log)'
COLLAB = ['the service process', 'scratch embedded store',
          'served frontend driven by playwright-bdd in a real browser',
          'per-provider stub-llm instances']

rows = [
    ('The operator switches the system default without restarting', LMC, 11,
     'S1-R1', 'the next reply uses the new default with no restart',
     'SCN-RC6-09: changing the default through any other route is rejected by the oracle'),
    ('A conversation without an override follows the new default and says so', LMC, 26,
     'S1-R2', 'a conversation with no override answers on the new default and shows its origin',
     'SCN-RC6-03: a conversation with an override must not follow the default'),
    ('A conversation override keeps answering on its own model', LMC, 41,
     'S1-R3', 'an explicit conversation choice keeps winning',
     'SCN-RC6-02: the default must not be served to the overridden conversation'),
    ('Clearing an override shows what the conversation will inherit', LMC, 51,
     'S1-R4', 'clearing previews and then uses the inherited default',
     'the previous override model must not serve the next turn'),
    ('Changing one conversation leaves the others alone', LMC, 66,
     'S1-R5', 'one conversation change does not move another',
     'the untouched conversation must keep its model'),
    ('A reply already being written finishes on the model it started with', LMC, 90,
     'S1-R6', 'an in-flight turn is not switched mid-reply',
     'the new default must not appear in the in-flight request'),
    ('A choice that cannot be served is refused and nothing is claimed', LMC, 108,
     'S1-R7', 'an unusable choice is refused with a reason and the previous value stays',
     'this scenario is itself the refusal control'),
    ('An agent with its own model keeps it and is listed when the default changes', LMC, 125,
     'S1-R8', 'agents with their own model keep it',
     'the pinned agent must not be served the new default'),
    ('Changing the default uses only the live route', LMC, 143,
     'S1-R1', 'the console changes the default only through the acknowledged live route',
     'this scenario is itself the wrong-route control'),
    ('The receipt names the model the provider reported, not the one requested', LMC, 150,
     'S1-R7', 'each reply shows the model that actually served it',
     'the requested-but-not-served model must not be shown'),
    ('Renaming a thread from its row menu keeps the new name after a reload', THR, 6,
     'thread menu', 'a renamed thread keeps its name after a reload',
     'SCN-RC6-12 (CTL-THR-001-CANCEL): cancelling keeps the old title'),
    ('Cancelling the rename dialog keeps the thread\'s title', THR, 16,
     'thread menu', 'cancel changes nothing', 'this scenario is itself the cancel control'),
    ('Deleting a thread from its row menu removes it for good and keeps the others', THR, 24,
     'thread menu', 'a deleted thread leaves the list and the others stay',
     'SCN-RC6-14 (CTL-THR-002-CANCEL): cancelling keeps the thread'),
    ('Cancelling the delete confirmation keeps the thread', THR, 41,
     'thread menu', 'cancel changes nothing', 'this scenario is itself the cancel control'),
    ('Every new conversation in one page load gets its own title', THR, 50,
     'thread menu', 'each new conversation gets its own title',
     'a placeholder title must not overwrite a user title'),
]

THREAD_SCOPE = ('rc.6 thread menu: rename and delete from the row menu persist after a reload, cancel changes '
                'nothing, and each new conversation gets its own title')
scope_items = [s for s in c['functionalScope'] if s.startswith('S1-R')]
assert len(scope_items) == 9, len(scope_items)
withdrawn_scope = [s for s in c['functionalScope'] if not s.startswith('S1-R')]
c['functionalScope'] = scope_items + [THREAD_SCOPE]


def scope_for(rule: str) -> str:
    if rule == 'thread menu':
        return THREAD_SCOPE
    hits = [s for s in scope_items if s.startswith(rule + ' ')]
    assert len(hits) == 1, (rule, hits)
    return hits[0]


SRC = {
    'S1-R1': 'openspec:provider-model-settings-certification#The system default model is the registry default provider plus its default model',
    'S1-R2': 'openspec:session-configuration#Each answer shows the model that served it, why and since when',
    'S1-R3': 'openspec:session-configuration#An explicit model beats the system default',
    'S1-R4': 'openspec:session-configuration#Clearing a conversation override previews the inherited result',
    'S1-R5': "openspec:session-configuration#One conversation's model choice does not change another",
    'S1-R6': 'openspec:session-configuration#A turn already generating finishes on the model it started with',
    'S1-R7': 'openspec:session-configuration#A model choice that cannot be served is refused with a reason',
    'S1-R8': 'openspec:session-configuration#The console shows which agents keep an explicit model when the default changes',
    'S1-R9': 'openspec:session-configuration#Each answer shows the model that served it, why and since when',
    'thread menu': ('assumption: each sidebar thread row menu renames and deletes its conversation (operator grant '
                    '2026-11-22, review/repair-round-4-proposal.md section 2)'),
}

rows[9] = rows[9][:3] + ('S1-R9',) + rows[9][4:]

# Registered owner tasks, taken from the withdrawn scenario that proved the same rule.
old_owner = {s['id']: s['ownerTask'] for s in c['scenarioMatrix']}
OWNER_BY_RULE = {'S1-R1': 'SCN-SYS-001', 'S1-R2': 'SCN-SYS-015', 'S1-R3': 'SCN-SYS-011',
                 'S1-R4': 'SCN-SYS-012', 'S1-R5': 'SCN-SYS-013', 'S1-R6': 'SCN-SYS-014',
                 'S1-R7': 'SCN-SYS-016', 'S1-R8': 'SCN-SYS-003', 'S1-R9': 'SCN-SYS-017',
                 'thread menu': 'SCN-THR-001'}


def owner_for(index: int, rule: str) -> str:
    if index == 9:  # SCN-RC6-09, live route only, was SCN-SYS-007
        return old_owner['SCN-SYS-007']
    return old_owner[OWNER_BY_RULE[rule]]

matrix = []
for i, (name, feature, line, rule, outcome, control) in enumerate(rows, 1):
    matrix.append({
        'id': f'SCN-RC6-{i:02d}',
        'name': name,
        'actor': 'console operator' if feature == LMC else 'chat user',
        'usefulOutcome': outcome,
        'sourceRequirement': SRC[rule],
        'reasonNow': 'code new since rc.5 (operator, 2026-11-22: validate only new code to exit rc.6)',
        'likelihood': 'UNKNOWN',
        'mandatory': True,
        'scopeRefs': [scope_for(rule)],
        'ownerTask': owner_for(i, rule),
        'productionEntry': f'served console in a real browser; {feature} (generated spec line {line})',
        'realCollaborators': COLLAB,
        'substitutedBoundaries': ['model vendor (stub-llm)'],
        'oracle': {
            'observation': 'the Playwright-BDD step assertions of this scenario: stub request log and visible console state',
            'expectedTruthSource': feature,
            'comparator': 'scenario passes only if every step assertion holds; --retries 0',
        },
        'negativeControl': {'input': control, 'expectedRejection': 'the scenario fails',
                            'commandRef': 'rc6-bdd-controls'},
        'fixtures': 'scratch data root, ephemeral ports, stub-llm fixtures under tests/bdd/fixtures',
        'evidence': EVID,
        'limitations': ['one replica', 'run on commit 8e4ce275; product code identical at 84840af1'],
        'invalidationKeys': [feature, 'tests/bdd/steps/**', 'tests/bdd/support/**', 'frontend/src/**', 'src/**'],
        'restoration': 'stop the services and delete the scratch root',
        'nondeterministic': True,
        'stochastic': {'samples': 1, 'grader': 'single run, --retries 0',
                       'stoppingRule': 'operator rule: BDD/VLBV once, no repeat sampling'},
    })

old_ids = [s['id'] for s in c['scenarioMatrix']]
c['scenarioMatrix'] = matrix

bp = c['boundaryPacket']
ids = [s['id'] for s in matrix]
RUN_NOTE = ('Run once on 2026-11-22 at 8e4ce275 (23:18:16Z, 242.5 s, 15 passed). The exact argv was not '
            'recorded; this entry names the feature files and flags stated in the run log and decision log.')
bp['commands'] = [
    {'id': 'rc6-bdd-new-code', 'kind': 'scenario', 'scenarioIds': ids, 'cwd': '.',
     'argv': ['playwright-bdd', LMC, THR, '--retries=0', '--workers=1'], 'note': RUN_NOTE},
    {'id': 'rc6-bdd-controls', 'kind': 'negativeControl', 'scenarioIds': ids, 'cwd': '.',
     'argv': ['playwright-bdd', LMC, THR, '--retries=0', '--workers=1'],
     'note': ('The controls are scenarios in the same single run (refusal, wrong route, cancel rename, cancel '
              'delete) and the negative assertions inside each scenario; no separate mutant build. ' + RUN_NOTE)},
]

safety = {s['id']: s for s in c['requiredSafetyChecks']}
safety['SAFE-NO-TEST-CREDENTIALS']['scenarioIds'] = ids
s = safety['SAFE-SETTINGS-API-CANNOT-WRITE-DEFAULT']
s['scenarioIds'] = ['SCN-RC6-09']
s['description'] += (' Superseding exit limit: SCN-RC6-09 proves the console writes the default only through the '
                     'live route and writes none of llm.default_provider, llm.model or run_policy.global. The '
                     'server-side rejection of a direct settings write (former SCN-SET-003) is not exercised by '
                     'this exit and moves to issue #542.')
safety['SAFE-REFUSAL-NEVER-CLAIMS-SERVED']['scenarioIds'] = ['SCN-RC6-07', 'SCN-RC6-10']
bp['receipts'] = [f'{PH}/rc6-check/3-bdd.json', f'{PH}/rc6-check/3-bdd-run.log', f'{PH}/boundary-results.json']
bp['requiredApprovals'] = ('operator instruction 2026-11-22 "Limit validation to ONLY new code to exit rc.6 work" '
                           'and "Run it"; superseding exit signed by the owner 2026-11-23')
bp['runnerOutputGlobs'] = [f'{PH}/rc6-check/**']
c['status']['implementationComplete'] = True

c['exclusions'].append(
    'superseding exit: the S1 packet (54 commands, mutant controls, baseline overlays, 3x repeats) and its '
    f'scenarios {",".join(old_ids)} are withdrawn from this phase; four attempts under boundary/attempt-* are '
    'kept as history and are not evidence of PASS. Unshipped goals move to issue #542 (rc.7).')
for item in withdrawn_scope:
    c['exclusions'].append('withdrawn by the superseding exit, not exercised, moved to issue #542: ' + item)

c['amendmentHistory'].append({
    'date': '2026-11-23',
    'supersedesDigest': prev,
    'reason': ('Operator: "Sign a superseding exit". Replaces the unfinished S1 boundary packet with the rc.6 '
               'new-code-only check the operator chose on 2026-11-22.'),
    'changes': [
        'scenarioMatrix: 15 SCN-RC6 scenarios (live-model-choice 10, thread-menu 5) replace 15 SCN rows',
        'boundaryPacket: one command, rc6-bdd-new-code, run once with --retries 0',
        'budget accounting covers the superseding packet only; consumption of the four earlier attempts was '
        'never metered and is not claimed within any ceiling',
        'status.implementationComplete: true (S1 shipped in v1.4.0-rc.6)'],
    'unchanged': 'budgets ceilings, history, earlier amendments and boundary attempt records',
    'operatorApproval': 'signed by the owner'})

json.dump(c, open(p, 'w'), indent=2, ensure_ascii=False)
open(p, 'a').write('\n')
print('contract edited:', p, 'scenarios:', len(matrix))
