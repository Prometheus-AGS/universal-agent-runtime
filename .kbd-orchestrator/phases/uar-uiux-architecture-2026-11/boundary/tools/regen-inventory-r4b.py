"""Regenerate the S1 inventory for the round 4b candidate at HEAD (be647d8c).

Usage: regen-inventory-r4.py <output path>. Run after amendment 2026-11-22-repair-round-4 is adopted.
"""
import json
import subprocess
import sys

P = '.kbd-orchestrator/phases/uar-uiux-architecture-2026-11'


def git(*a):
    return subprocess.check_output(['git', *a], text=True).strip()


inv = json.load(open(f'{P}/implementation-complete-inventory.json'))
assert inv['candidateCommit'].startswith('f129bbfb'), inv['candidateCommit']
contract = json.load(open(f'{P}/adopted-contract.json'))
assert contract['identity']['contractDigest'].startswith('a6b514f2'), contract['identity']['contractDigest']
assert git('log', '-1', '--format=%s').startswith('test(bdd): send a message the baseline stub answers'), 'HEAD is not be647d8c'

inv['supersededCandidates'].append({
    'commit': inv['candidateCommit'],
    'reason': 'Never run. The inventory critic found the THR scenarios sent a message the baseline stub cannot '
              'answer; be647d8c changes only thread-menu.feature (amendment 4b, contract a6b514f2).'})
inv['candidateCommit'] = git('rev-parse', 'HEAD')
inv['candidateCommittedAt'] = git('show', '-s', '--format=%cI', 'HEAD')
inv['contractDigest'] = contract['identity']['contractDigest']
inv['leadFixes'].append('be647d8c: thread-menu.feature sends "Reply with OK." so the THR negative controls reach the menu on the baseline.')
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
