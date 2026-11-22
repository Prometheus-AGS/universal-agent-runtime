"""Edit a copy or the working S1 contract for the combined rc.6 candidate.

Usage: rc6-candidate.py <contract.json>. Every edit asserts the text it replaces.
"""
import json
import sys

p = sys.argv[1]
c = json.load(open(p))
assert c['identity']['contractDigest'].startswith('03de9cf8'), c['identity']['contractDigest']
bp = c['boundaryPacket']

old = 'S1 changed no frontend dependency, lockfile or bundler configuration'
new = ('the combined rc.6 candidate takes the frontend dependency updates of main (#528) and its regenerated '
       'lockfile; S1 itself changed none, and both exemptions keep their exact limits below')
hits = [i for i, s in enumerate(bp['stopConditions']) if old in s]
assert len(hits) == 1, hits
bp['stopConditions'][hits[0]] = bp['stopConditions'][hits[0]].replace(old, new)

assert bp['expectedEffects'][10].startswith('baseline-check-bdd and baseline-check-supporting')
bp['expectedEffects'][10] += (
    ' Owner-accepted limit for the rc.6 candidate: the frozen expectation calls SCN-SYS-012 FAIL, but on f2c9e36 '
    'it fails in its second Given, because the old console lacks session-config-model-trigger, model-option and '
    'session-config-clear. It is NOT_RUNNABLE in this vocabulary. Its baseline receipt is never cited as an oracle '
    'rejecting the old build. The frozen file is unchanged and the comparison (a failing test) is unchanged.')

assert bp['expectedEffects'][28].startswith('Repair rule:')
bp['expectedEffects'][28] += (
    ' Run order for the combined rc.6 candidate: 0, 1, 7, 8, 23, 9-22, 24-49. scn-sys-001 runs after both baseline '
    'checks and before the mutant builds, so that a broken chat path stops the batch before those builds. Commands '
    '2-6 carry forward, because the baseline commit f2c9e36 is unchanged. The binding, the first-FAIL stop and '
    'every command are unchanged.')

c['exclusions'].append(
    'the combined rc.6 candidate also contains main at 0ce227af, issue #539 (supervised launch and the version '
    'route) and the image build fixes; an S1 pass certifies none of them. #539 keeps its own evidence and each '
    'image needs its own qualification')

b = c['budgets']
assert (b['repairAttempts'], b['timeMinutes']) == (3, 600), b
b['timeMinutes'] = 900

c['amendmentHistory'].append({
    'date': '2026-11-22',
    'supersedesDigest': c['identity']['contractDigest'],
    'reason': 'The operator chose one combined local rc.6 candidate ("Yes, combine them").',
    'changes': [
        'ruled frontend notice clause states the merged dependency updates',
        'SCN-SYS-012 baseline limit accepted by the owner; frozen file unchanged',
        'run order 0,1,7,8,23,9-22,24-49 for this candidate',
        'exclusion: S1 certifies neither main, #539 nor the images',
        'budgets.timeMinutes 600 to 900, cumulative (at least 281 already recorded)'],
    'unchanged': 'scenarioMatrix, feature files, oracles, controls, verdict rules, commands, BIND checks, frozen '
                 'fixtures, repairAttempts; consumed budgets and history are not reset',
    'operatorApproval': 'signed by the owner'})
json.dump(c, open(p, 'w'), indent=2, ensure_ascii=False)
open(p, 'a').write('\n')
print('contract edited:', p)
