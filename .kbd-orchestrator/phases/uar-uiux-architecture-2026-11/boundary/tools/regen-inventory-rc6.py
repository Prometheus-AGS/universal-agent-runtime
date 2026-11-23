"""Regenerate the S1 inventory for the rc.6 candidate at HEAD.

Usage: regen-inventory-rc6.py <output path>. Run after the amendment is adopted.
"""
import json
import subprocess
import sys

P = '.kbd-orchestrator/phases/uar-uiux-architecture-2026-11'
def git(*a):
    return subprocess.check_output(['git', *a], text=True).strip()

inv = json.load(open(f'{P}/implementation-complete-inventory.json'))
assert inv['candidateCommit'].startswith('c8eb9e23'), inv['candidateCommit']
contract = json.load(open(f'{P}/adopted-contract.json'))

inv['supersededCandidates'].append({
    'commit': inv['candidateCommit'],
    'reason': 'Boundary attempt 3 (archived in boundary/attempt-3-c8eb9e23): 1 PASS, 2 FAIL, 10 BLOCKED. Repair round 3 followed. '
              'Correction to the ae01b21c entry, which names c172da51 for all of attempt 1: per decision-log.md commands 0-1 '
              'ran under 32717a4e, 2-6 under 41a887fb, and 7 passed under c172da51 (after failures under 41a887fb and '
              '911b6abd). Only command 8 and the passing run of 7 used c172da51.'})
inv['candidateCommit'] = git('rev-parse', 'HEAD')
inv['candidateCommittedAt'] = git('show', '-s', '--format=%cI', 'HEAD')
inv['contractDigest'] = contract['identity']['contractDigest']
inv['leadFixes'].append(
    'B5 repair round 3 (d7c8db9e, review/round3/diff-r5.patch, re-review PASS): picker windowing, PGlite migrations, '
    'burst 100, BDD local owner and scratch config, provider default switch keeps verification proofs, baseline '
    'teardown. Then the combined rc.6 candidate: main 0ce227af, image fixes 46970b46, #539 72894dfa, version bump.')
inv['unverified'] = (
    'No scenario or control has a verdict on this candidate. Plan: 0, 1, 7, 8, 23, 9-22, 24-49; commands 2-6 carry '
    'forward (baseline f2c9e36 unchanged); both mutants are removed and rebuilt from this commit. Limits: '
    '(1) the round 3 causes were fixed from source reading and nine of the ten chat scenarios have never run past setup; '
    '(2) the baseline BDD check shows that the old build lacks the feature, not that an oracle rejects it; SCN-SYS-012 '
    'fails there in its Given; (3) IDN-001 uniqueness sees only mounted rows; (4) PICK-002 empties configured models '
    'and does not time the dialog opening; (5) SYS-014 depends on a 15 s stream-start timeout; (6) the BDD burst of '
    '100 is stated, so BDD cannot detect a regression of that default; (7) parent environment and home skill '
    'directories are not hermetic beyond the blanked keys; (8) static/ is outside the binding and is rebuilt by '
    'prepare-bdd; (9) product-screen-validation cannot pass on the built page and no command selects it; '
    '(10) the environment-key case of the proof fix has no scenario and is checked at image qualification; '
    '(11) main, #539 and the images are not certified by S1. Hooks compiled only the minimal feature set.')
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
