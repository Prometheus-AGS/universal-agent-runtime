"""Write the operator exit details for `kbd-scope-authority-template.sh supersede-exit`.

Usage: supersede-exit-details.py <rc6-rows.json> <out.json>
<rc6-rows.json> holds [{id, name, scopeRefs}] for the 15 rc.6 checks. Verdicts come
only from rc6-check/3-bdd.json (one run on 8e4ce275, --retries 0); nothing is re-run.
Every bound scenario and every bound scope item not exercised is withdrawn to #542.
"""
import hashlib
import json
import sys

PH = '.kbd-orchestrator/phases/uar-uiux-architecture-2026-11'
ISSUE = 'https://github.com/Prometheus-AGS/universal-agent-runtime/issues/542'
FEATURES = {
    'live-model-choice': 'tests/bdd/features/live-model-choice.feature',
    'thread-menu': 'tests/bdd/features/thread-menu.feature',
}

rows_path, out_path = sys.argv[1:3]
rows = json.load(open(rows_path))
bound = json.load(open(f'{PH}/adopted/a6b514f2c0b0cbd2d6f6d14d6ff58588f13fce5a957cc22f928ce916c240d0f9.json'))
report = json.load(open(f'{PH}/rc6-check/3-bdd.json'))


def sha(path: str) -> str:
    return hashlib.sha256(open(path, 'rb').read()).hexdigest()


def ev(path: str) -> dict:
    return {'path': path, 'sha256': sha(path)}


results = {}
for suite in report['suites']:
    stack = [suite]
    while stack:
        s = stack.pop()
        stack.extend(s.get('suites', []))
        for spec in s.get('specs', []):
            r = spec['tests'][0]['results']
            assert len(r) == 1, (spec['title'], len(r))  # one run, no retries
            results[spec['title']] = (r[0]['status'], spec['file'])

bound_scope = set(bound['functionalScope'])
exercised, covered = [], set()
for row in rows:
    status, file = results[row['name']]
    feature = next(p for k, p in FEATURES.items() if k in file)
    refs = [r for r in row['scopeRefs'] if r in bound_scope]
    covered.update(refs)
    item = {
        'id': row['id'],
        'verdict': 'PASS' if status == 'passed' else 'FAIL',
        'evidence': [ev(feature), ev(f'{PH}/rc6-check/3-bdd.json'), ev(f'{PH}/rc6-check/3-bdd-run.log')],
    }
    if refs:
        item['scopeRefs'] = refs
    exercised.append(item)
assert len(exercised) == 15

withdrawn = [{'id': s['id'], 'kind': 'scenario', 'destination': ISSUE} for s in bound['scenarioMatrix']]
withdrawn += [{'id': s, 'kind': 'scope', 'destination': ISSUE}
              for s in bound['functionalScope'] if s not in covered]

details = {
    'operatorDecision': {
        'reference': 'operator instructions in session f1444227, 2026-11-23; '
                     f'{PH}/reflection.md (Delta and Next Phase Seed)',
        'text': '"Sign a superseding exit" and "Yes, close it": the rc.6 new-code-only BDD check '
                '(15 scenarios, run once on 8e4ce275, released as 84840af1) replaces the unfinished '
                'S1 boundary packet; unshipped scope moves to issue #542.',
    },
    'exercised': exercised,
    'withdrawn': withdrawn,
    'budgetAccounting': {
        'class': 'measured',
        'values': {
            'durationMilliseconds': int(report['stats']['duration']),
            'boundaryCommands': 1,
            'repairAttempts': 0,
            'retries': 0,
        },
    },
    'limits': [
        'None of the 15 bound S1 scenarios is certified; each is withdrawn to #542.',
        'The rc.6 checks ran once on 8e4ce275; the released 84840af1 has identical tests/bdd, '
        'frontend/src and src, but was not re-run.',
        'Server-side rejection of a direct settings-API write to the default is not exercised.',
        'The four earlier boundary attempts were not metered and their verdicts are not claimed.',
        'Thread-menu checks (SCN-RC6-11..15) cover rc.6 scope added outside the bound contract.',
    ],
}
json.dump(details, open(out_path, 'w'), indent=2)
print('exercised', len(exercised), 'PASS', sum(e['verdict'] == 'PASS' for e in exercised),
      '| withdrawn', len(withdrawn), '| scope covered', len(covered))
