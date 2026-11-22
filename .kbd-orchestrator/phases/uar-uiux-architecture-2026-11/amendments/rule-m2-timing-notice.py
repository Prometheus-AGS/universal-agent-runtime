"""Exempt one bundler timing notice in one receipt of boundary attempt 4.

Usage: rule-m2-timing-notice.py <contract.json>.
"""
import json
import sys

p = sys.argv[1]
c = json.load(open(p))
assert c['identity']['contractDigest'].startswith('45f441e0'), c['identity']['contractDigest']
bp = c['boundaryPacket']

anchor = 'any other chunk over the limit or any other eval location is not exempt'
hits = [i for i, s in enumerate(bp['stopConditions']) if anchor in s]
assert len(hits) == 1, hits
bp['stopConditions'][hits[0]] += (
    ' One-receipt exception (operator ruling 2026-11-22): in the build-mutant-m2 receipt of candidate 9ad39030 that '
    'ended 2026-11-22T10:13:54-0600 (exit 0), the rolldown notice `[PLUGIN_TIMINGS] JavaScript callbacks ran for 6.8s '
    'of this 8.9s build (76%).` is exempt. It reports elapsed time under machine load; the page that build produced '
    'is byte-identical to the candidate and M1 pages. The notice is exempt in no other receipt. The run resumes at '
    'command 24 and command 22 is not rebuilt.')

c['amendmentHistory'].append({
    'date': '2026-11-22',
    'supersedesDigest': c['identity']['contractDigest'],
    'reason': 'Boundary attempt 4 stopped at build-mutant-m2 on a rolldown timing notice with exit 0.',
    'changes': ['one-receipt exception for that notice; resume at command 24 without rebuilding'],
    'unchanged': 'scenarioMatrix, oracles, controls, verdict rules, commands, runner, BIND checks, frozen fixtures, '
                 'candidate, budgets and every other warning rule; history is not reset',
    'operatorApproval': 'signed by the owner'})
json.dump(c, open(p, 'w'), indent=2, ensure_ascii=False)
open(p, 'a').write('\n')
print('contract edited:', p)
