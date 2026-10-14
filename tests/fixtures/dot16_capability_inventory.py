"""Installed-host discovery helper for the full-phase runner. Not self-executing.

Each available transport entry must have a caller-provided real-path scenario.
The caller owns test fixtures, authenticated principals, process lifecycle and
independent effect observations. A discovery response alone cannot pass a case.
"""
import json
import urllib.request


def inventory(base_url, bearer):
    request = urllib.request.Request(
        base_url.rstrip('/') + '/api/uar/assistant-capabilities',
        headers={'Authorization': 'Bearer ' + bearer})
    with urllib.request.urlopen(request, timeout=30) as response:
        value = json.load(response)
    assert value['schema_version'] == 1, value
    identifiers = [operation['id'] for operation in value['operations']]
    assert len(set(identifiers)) == len(identifiers), identifiers
    assert {'memory.policy.read', 'schedule.create', 'followup.inbox',
            'tool.outcomes', 'approval.decide', 'checkpoint.resume'} <= set(identifiers)
    for operation in value['operations']:
        for entry in operation['entry_points']:
            if entry['available']:
                assert entry['target'], (operation['id'], entry)
            else:
                assert entry['reason'], (operation['id'], entry)
            if entry['transport'] == 'direct_embedded_runtime':
                assert not entry['available'], entry
    return value


def exercise_inventory(value, scenarios, unsupported_scenarios):
    """Scenarios return observed evidence or raise; no skipped case is a pass."""
    receipts = []
    for operation in value['operations']:
        for entry in operation['entry_points']:
            key = (operation['id'], entry['transport'])
            if entry['available']:
                scenario = scenarios[key]
            elif entry['transport'] == 'http':
                scenario = unsupported_scenarios[key]
            else:
                continue  # No nonexistent tool/embedded method to invoke.
            evidence = scenario(operation, entry)
            assert isinstance(evidence, dict) and evidence.get('observations'), key
            receipts.append({'operation': key[0], 'transport': key[1],
                             'available': entry['available'], 'evidence': evidence})
    return receipts


def inspect_outcomes(base_url, owner_token, colliding_subject_token, run_id, expected):
    """Use after real tool execution, then repeat after a cold process restart.

    expected maps independently observed tool-call IDs to expected outcomes.
    The second token has the same subject in a different verified issuer/tenant.
    """
    def read(token):
        request = urllib.request.Request(
            base_url.rstrip('/') + '/api/uar/runs/' + run_id + '/tool-outcomes',
            headers={'Authorization': 'Bearer ' + token})
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)['receipts']
    records = read(owner_token)
    assert {record['call_id']: record['outcome'] for record in records} == expected, records
    assert all(not ({'typed_value', 'raw_segments', 'owner_id', 'principal_key'} & set(record))
               for record in records), records
    assert read(colliding_subject_token) == [], 'Cross-principal receipt disclosure'
    return {'observations': records, 'colliding_principal_receipts': []}
