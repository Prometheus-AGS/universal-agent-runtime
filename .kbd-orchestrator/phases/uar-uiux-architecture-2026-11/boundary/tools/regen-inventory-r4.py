"""Regenerate the S1 inventory for the repair round 4 candidate at HEAD.

Usage: regen-inventory-r4.py <output path>. Run after amendment 2026-11-22-repair-round-4 is adopted.
"""
import json
import subprocess
import sys

P = '.kbd-orchestrator/phases/uar-uiux-architecture-2026-11'


def git(*a):
    return subprocess.check_output(['git', *a], text=True).strip()


inv = json.load(open(f'{P}/implementation-complete-inventory.json'))
assert inv['candidateCommit'].startswith('9ad39030'), inv['candidateCommit']
contract = json.load(open(f'{P}/adopted-contract.json'))
assert contract['identity']['contractDigest'].startswith('d5bb7921'), contract['identity']['contractDigest']
assert git('log', '-1', '--format=%s').startswith('fix: repair round 4'), 'HEAD is not the round 4 commit'

inv['supersededCandidates'].append({
    'commit': inv['candidateCommit'],
    'reason': 'Boundary attempt 4 (boundary/attempt-4-verdicts.md): 2 PASS (provisional), 6 FAIL, 5 NOT_EXERCISED; '
              'ended during command 38. Repair round 4 followed; every attempt-4 verdict is void.'})
inv['candidateCommit'] = git('rev-parse', 'HEAD')
inv['candidateCommittedAt'] = git('show', '-s', '--format=%cI', 'HEAD')
inv['contractDigest'] = contract['identity']['contractDigest']
inv['leadFixes'].append(
    'B5 repair round 4 (f129bbfb, review/round4/diff-r3.patch; reviews in review/round4/review-record.md): thread '
    'menu onClick and dialogs that close after the PGlite write; composer busy until the stream lease is released '
    'and Stop inert after the reply; harness reply counter, virtual picker scroll, complete-reply read, agent list '
    'item selection; new SCN-THR-001 and SCN-THR-002.')
inv['unverified'] = (
    'No scenario or control has a verdict on this candidate. Plan: 0, 1, 7, 8, 23, 49-52, 9-22, 24-48, 53; commands '
    '2-6 carry forward (baseline f2c9e36 unchanged); both mutants are removed and rebuilt from this commit. Limits '
    'carried from the rc.6 inventory: (1) nine of the ten chat scenarios have never reached their own assertions; '
    '(2) the baseline BDD check shows the old build lacks the feature, not that an oracle rejects it; SCN-SYS-012 '
    'fails there in its Given; (3) IDN-001 uniqueness sees only mounted rows; (4) PICK-002 empties configured models '
    'and does not time the dialog opening; (5) SYS-014 depends on a 15 s stream-start timeout; (6) the BDD burst of '
    '100 is stated; (7) parent environment and home skill directories are not hermetic beyond the blanked keys; '
    '(8) static/ is outside the binding and is rebuilt by prepare-bdd; (9) product-screen-validation cannot pass on '
    'the built page and no command selects it; (10) the environment-key case of the proof fix is checked at image '
    'qualification; (11) main, #539 and the images are not certified by S1. New in round 4: (12) THR reload proves '
    'the browser database on the embedded store only (P2, D1 follow-ups); (13) neg-thr-002 fails on the baseline at '
    'its Rename setup, not at Delete; (14) a late generated title overwriting a rename is a residual timing risk; '
    '(15) the P1 busy flag is per thread while the lease is global, and A2UI lease holders do not set it.')
rows = git('-c', 'core.quotePath=false', 'diff', '--name-status', 'f2c9e36', 'HEAD', '--', '.',
           ':(exclude,glob)**/.kbd-orchestrator/**', ':(exclude,glob)**/.prometheus/**').splitlines()
paths = []
for row in rows:
    status, *names = row.split('\t')
    entry = {'status': status[0], 'path': names[-1]}
    if len(names) == 2:
        entry['from'] = names[0]
    paths.append(entry)
inv['changedPaths'] = paths
json.dump(inv, open(sys.argv[1], 'w'), indent=2, ensure_ascii=False)
open(sys.argv[1], 'a').write('\n')
print(inv['candidateCommit'][:8], inv['contractDigest'][:8], len(paths), 'changed paths')
