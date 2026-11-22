"""Re-freeze thread-menu.feature after the inventory critic's finding (2026-11-22).

The baseline stub reads only live-model-choice.json, which has no fixture for "What is 2 plus 2?", so on the
baseline both THR scenarios would fail at their first send and the negative controls would prove nothing. The
feature now sends "Reply with OK.", which both stubs answer. Only the frozen digest of that one file changes.

Usage: repair-round-4b.py <contract.json>. Every edit asserts what it replaces.
"""
import hashlib
import json
import sys

FEATURE = 'tests/bdd/features/thread-menu.feature'
OLD = 'dedb2b0f2876339a935c1c0c71a8439e5ca6d1c08bc1dc7257476962f1868046'
p = sys.argv[1]
c = json.load(open(p))
assert c['identity']['contractDigest'].startswith('d5bb7921'), c['identity']['contractDigest']
new = hashlib.sha256(open(FEATURE, 'rb').read()).hexdigest()
assert new != OLD

text = json.dumps(c, ensure_ascii=False)
hits = text.count(OLD)
assert hits > 0, 'old feature digest not found'
OLD_FIX = 'stub fixture tests/bdd/fixtures/bdd-chat.json (\\"What is 2 plus 2?\\")'
NEW_FIX = 'stub fixture tests/bdd/fixtures/live-model-choice.json (\\"Reply with OK.\\", answered by the baseline stub too)'
assert text.count(OLD_FIX) == 2, text.count(OLD_FIX)
c = json.loads(text.replace(OLD, new).replace(OLD_FIX, NEW_FIX))
c['amendmentHistory'].append({
    'date': '2026-11-22', 'supersedesDigest': c['identity']['contractDigest'],
    'reason': ('Inventory critic finding: the baseline stub cannot answer "What is 2 plus 2?", so the THR '
               'negative controls would fail at the first send. thread-menu.feature now sends "Reply with OK.".'),
    'changes': [f'frozen digest of {FEATURE} {OLD[:12]} to {new[:12]} ({hits} occurrences)',
                'THR fixtures text names the live-model-choice.json message'],
    'unchanged': 'everything else, including every command body apart from that digest',
    'operatorApproval': 'signed by the owner'})
json.dump(c, open(p, 'w'), indent=2, ensure_ascii=False)
open(p, 'a').write('\n')
print('contract edited:', p, hits, 'occurrences', new[:12])
