"""Write boundary-results.json for the superseding exit from the recorded rc.6 BDD run.

Usage: superseding-exit-evidence.py <bound-contract-digest> <project-id> <run-id>
Every verdict comes from rc6-check/3-bdd.json; nothing is re-run or inferred.
"""
import hashlib
import json
import sys

digest, project, run = sys.argv[1:4]
PH = '.kbd-orchestrator/phases/uar-uiux-architecture-2026-11'
contract = json.load(open(f'{PH}/adopted-contract.json'))
report = json.load(open(f'{PH}/rc6-check/3-bdd.json'))


def sha(path: str) -> str:
    return hashlib.sha256(open(path, 'rb').read()).hexdigest()


results = {}
for suite in report['suites']:
    stack = [suite]
    while stack:
        s = stack.pop()
        stack.extend(s.get('suites', []))
        for spec in s.get('specs', []):
            r = spec['tests'][0]['results']
            assert len(r) == 1, (spec['title'], len(r))  # run once, no retries
            results[spec['title']] = (r[0]['status'], r[0]['duration'], spec['file'], spec['line'])

scenarios = []
for row in contract['scenarioMatrix']:
    status, ms, file, line = results[row['name']]
    scenarios.append({
        'id': row['id'],
        'status': 'PASS' if status == 'passed' else 'FAIL',
        'attempted': f'{file}:{line} once on 8e4ce275, --retries 0',
        'observed': f'playwright status {status} in {ms} ms',
        'evidence': f'{PH}/rc6-check/3-bdd.json; {PH}/rc6-check/3-bdd-run.log',
    })
assert len(scenarios) == 15

files = ['tests/bdd/features/live-model-choice.feature', 'tests/bdd/features/thread-menu.feature',
         f'{PH}/rc6-check/3-bdd.json', f'{PH}/rc6-check/3-bdd-run.log']
out = {
    'phase': 'uar-uiux-architecture-2026-11',
    'projectId': project,
    'runId': run,
    'contractDigest': digest,
    'candidateCommit': '8e4ce27516ec0df8ab3544b07b0722f72ede0a97',
    'releasedCommit': '84840af1ebdddee2b84d165f3d3c9eaa17030e1e',
    'note': 'tests/bdd, frontend/src and src are identical between the two commits (git diff --quiet)',
    'sourceDigests': {f: sha(f) for f in files},
    'scenarios': scenarios,
    'budgetConsumption': {
        'source': ('superseding packet only: rc6-check/3-bdd.json stats (242.5 s), one boundary command, '
                   'no repair. The four earlier attempts were never metered and are not claimed.'),
        'values': {
            'timeMinutes': {'class': 'measured', 'value': round(report['stats']['duration'] / 60000, 2)},
            'toolCalls': {'class': 'measured', 'value': 1},
            'repairAttempts': {'class': 'measured', 'value': 0},
            'noProgress': {'class': 'measured', 'value': 0},
        },
    },
}
json.dump(out, open(f'{PH}/boundary-results.json', 'w'), indent=2)
print('wrote boundary-results.json:', sum(s['status'] == 'PASS' for s in scenarios), 'PASS of', len(scenarios))
