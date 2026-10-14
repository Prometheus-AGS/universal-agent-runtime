"""Fixed real HTTP operation drivers and strict cross-stage reconciliation.

No callback registry, arbitrary endpoint execution, supplied operation outcomes or
synthetic supervisor proofs. Only the full-phase acceptance entrypoint calls this.
"""
import datetime
import json
from pathlib import Path
import re
import urllib.parse
import uuid

from acceptance import Blocked, digest, poll, read, require
from journeys import decision, finish, instance, pending, segment, start_run, tool_input, turn

CONTRACTS = {
    'memory.policy.read': ('GET', '/api/memory/policy'),
    'memory.policy.update': ('PUT', '/api/memory/policy'),
    'memory.export': ('GET', '/api/memory/records'),
    'memory.read': ('GET', '/api/memory/records/{id}'),
    'memory.update': ('PATCH', '/api/memory/records/{id}'),
    'memory.delete': ('DELETE', '/api/memory/records/{id}'),
    'memory.history': ('GET', '/api/memory/records/{id}/history'),
    'memory.control': ('GET', '/api/memory/records/{id}/control'),
    'schedule.create': ('POST', '/api/uar/schedules/'),
    'schedule.list': ('GET', '/api/uar/schedules/'),
    'schedule.read': ('GET', '/api/uar/schedules/{id}'),
    'schedule.update': ('PUT', '/api/uar/schedules/{id}'),
    'schedule.pause': ('POST', '/api/uar/schedules/{id}/pause'),
    'schedule.resume': ('POST', '/api/uar/schedules/{id}/resume'),
    'schedule.cancel': ('POST', '/api/uar/schedules/{id}/cancel'),
    'followup.create': ('POST', '/api/uar/followups/'),
    'followup.list': ('GET', '/api/uar/followups/'),
    'followup.read': ('GET', '/api/uar/followups/{id}'),
    'followup.update': ('PUT', '/api/uar/followups/{id}'),
    'followup.enable': ('POST', '/api/uar/followups/{id}/enable'),
    'followup.disable': ('POST', '/api/uar/followups/{id}/disable'),
    'followup.revoke': ('POST', '/api/uar/followups/{id}/revoke'),
    'followup.inbox': ('GET', '/api/uar/followups/inbox/{conversation}'),
    'followup.acknowledge': ('POST', '/api/uar/followups/inbox/{conversation}/{item}/acknowledge'),
    'tool.outcomes': ('GET', '/api/uar/runs/{run_id}/tool-outcomes'),
    'approval.read': ('GET', '/api/uar/runs/{run_id}/tool-approval'),
    'approval.decide': ('POST', '/api/uar/runs/{run_id}/tool-approval'),
    'checkpoint.list': ('GET', '/api/uar/runs/{run_id}/checkpoints'),
    'checkpoint.resume': ('POST', '/api/uar/runs/{run_id}/resume/{checkpoint_id}'),
    'recovery.challenge': ('POST', '/api/uar/recovery/supervisor/challenges'),
    'recovery.exclude': ('POST', '/api/uar/recovery/supervisor/challenges/{id}/exclude'),
}
PROTECTED = {'checkpoint.resume', 'recovery.challenge', 'recovery.exclude'}


def matches(identifier, row):
    method, target = CONTRACTS[identifier]
    pattern = '^' + '[^/]+'.join(re.escape(part) for part in re.split(r'\{[^}]+\}', target)) + '$'
    principal = 'host' if identifier.startswith('recovery.') else 'owner'
    return (row.get('principal') == principal and row.get('method') == method
            and re.fullmatch(pattern, urllib.parse.urlsplit(row.get('path', '')).path)
            and 200 <= row.get('status', 0) < 300)


def inventory_http(inventory):
    require(inventory['schema_version'] == 1, 'unknown capability inventory schema')
    operations = inventory['operations']
    require(len(operations) == len(CONTRACTS) and {row['id'] for row in operations} == set(CONTRACTS),
            'advertised operation set changed; explicit HTTP driver update required')
    result = {}
    for operation in operations:
        entries = [entry for entry in operation['entry_points'] if entry['transport'] == 'http']
        require(len(entries) == 1, 'HTTP entry must be unique')
        entry = entries[0]
        require((entry['method'], entry['target']) == CONTRACTS[operation['id']], 'advertised HTTP route changed')
        require(entry['available'] or entry.get('reason'), 'unavailable operation lacks reason')
        for transport in operation['entry_points']:
            if transport['transport'] == 'direct_embedded_runtime':
                require(not transport['available'], 'unimplemented direct embedded capability advertised')
        result[operation['id']] = entry
    return result


def existing(evidence, host, identifiers):
    return {identifier: [index for index, row in enumerate(evidence.http)
                        if row.get('host') == host.label and matches(identifier, row)]
            for identifier in identifiers}


def memory_controls(host, evidence, missing):
    policy = host.api('GET', '/api/memory/policy')
    if 'memory.policy.update' in missing:
        changed = host.api('PUT', '/api/memory/policy',
                           {'enabled': policy['enabled'], 'retention': policy.get('retention')})
        require(changed['enabled'] == policy['enabled'] and changed['retention'] == policy.get('retention'),
                'policy roundtrip changed consent')
    marker = 'HTTP_CAP_MEMORY_' + uuid.uuid4().hex
    key = host.api('POST', '/api/memory', {'content': marker})['id']
    path = '/api/memory/records/' + segment(key)
    try:
        require(host.api('GET', path)['content'] == marker, 'actual memory read lost created content')
        if 'memory.export' in missing:
            require(any(row['content'] == marker for row in host.api('GET', '/api/memory/records')['memories']),
                    'memory export omitted owned record')
        if 'memory.control' in missing:
            control = host.api('GET', path + '/control')
            require(isinstance(control, dict) and control, 'memory control metadata absent')
        if 'memory.update' in missing:
            marker += '_UPDATED'
            host.api('PATCH', path, {'content': marker})
            require(host.api('GET', path)['content'] == marker, 'HTTP update did not persist')
        if 'memory.history' in missing:
            require(host.api('GET', path + '/history')['history'], 'memory history is empty')
        # Legacy compatible search is a required supplemental real operation,
        # although current inventory does not advertise a memory.search ID.
        found = poll(lambda: host.api('GET', '/api/memory?q=' + urllib.parse.quote(marker, safe='') + '&limit=10'),
                     lambda rows: any(row['content'] == marker for row in rows), 'actual memory search did not retrieve fixture')
        evidence.observe('http-memory-search', {'memory_id': key, 'matches': len(found)})
    finally:
        deleted = host.api('DELETE', path)
        require(deleted['deleted'] is True, 'memory deletion not recorded')
        host.api('GET', path, expect=404)


def schedule_controls(host, spec, missing):
    if missing == {'schedule.list'}:
        require(isinstance(host.api('GET', '/api/uar/schedules/'), list), 'schedule list shape')
        return
    target = instance(host, spec['deployment_binding_id'])
    request = {'instanceId': target['instanceId'], 'prompt': 'HTTP capability future schedule',
               'recurrence': {'kind': 'once', 'at': (datetime.datetime.now(datetime.timezone.utc)
                   + datetime.timedelta(days=1)).isoformat()}}
    row = host.api('POST', '/api/uar/schedules/', request, expect=201)
    path = '/api/uar/schedules/' + segment(row['id'])
    try:
        require(any(item['id'] == row['id'] for item in host.api('GET', '/api/uar/schedules/')), 'created schedule absent')
        row = host.api('GET', path)
        if 'schedule.update' in missing:
            request['prompt'] = 'HTTP capability revised future schedule'
            before = row['revision']
            row = host.api('PUT', path, {'expectedRevision': before, 'schedule': request})
            require(row['revision'] > before, 'schedule update did not commit revision')
        # Resume has a real paused prerequisite; a no-op response is insufficient.
        if {'schedule.pause', 'schedule.resume'} & missing:
            row = host.api('POST', path + '/pause', {'expectedRevision': row['revision']})
            require(row['paused'], 'schedule pause not committed')
            if 'schedule.resume' in missing:
                row = host.api('POST', path + '/resume', {'expectedRevision': row['revision']})
                require(not row['paused'], 'schedule resume not committed')
    finally:
        row = host.api('GET', path)
        row = host.api('POST', path + '/cancel', {'expectedRevision': row['revision']})
        require(row['cancelled'] and row['active'] is None, 'future schedule cleanup did not cancel')


def followup_controls(host, spec, missing):
    if missing == {'followup.list'}:
        require(isinstance(host.api('GET', '/api/uar/followups/'), list), 'followup list shape')
        return
    source = instance(host, spec['deployment_binding_id'])
    destination = instance(host, spec['deployment_binding_id'])
    request = {'sourceInstanceId': source['instanceId'], 'destinationInstanceId': destination['instanceId'],
               'conversationId': destination['sessionId'], 'eventKinds': ['turn_completed']}
    row = host.api('POST', '/api/uar/followups/', request, expect=201)
    path = '/api/uar/followups/' + segment(row['id'])
    inbox_path = '/api/uar/followups/inbox/' + segment(destination['sessionId'])
    try:
        require(not row['enabled'], 'followup creation silently enabled consent')
        require(any(item['id'] == row['id'] for item in host.api('GET', '/api/uar/followups/')), 'created rule absent')
        row = host.api('GET', path)
        if 'followup.update' in missing:
            before = row['revision']
            row = host.api('PUT', path, {'expectedRevision': before, 'rule': dict(request, enabled=False)})
            require(row['revision'] > before and not row['enabled'], 'followup update did not preserve disabled intent')
        if {'followup.enable', 'followup.disable', 'followup.inbox', 'followup.acknowledge'} & missing:
            row = host.api('POST', path + '/enable', {'expectedRevision': row['revision']})
            require(row['enabled'], 'followup consent not enabled')
        if {'followup.inbox', 'followup.acknowledge'} & missing:
            turn(host, source['instanceId'], json.dumps({'dot_action': 'text', 'marker': 'http-cap-followup-' + uuid.uuid4().hex}))
            items = poll(lambda: [item for item in host.api('GET', inbox_path) if item['ruleId'] == row['id']],
                         bool, 'actual source turn not delivered')
            require(len(items) == 1, 'duplicate capability followup delivery')
            host.api('POST', inbox_path + '/' + segment(items[0]['id']) + '/acknowledge', expect=204)
            require(next(item for item in host.api('GET', inbox_path) if item['id'] == items[0]['id'])['acknowledgedAt'],
                    'followup acknowledgement not durable')
        if 'followup.disable' in missing:
            row = host.api('GET', path)
            row = host.api('POST', path + '/disable', {'expectedRevision': row['revision']})
            require(not row['enabled'], 'followup disable not committed')
    finally:
        row = host.api('GET', path)
        row = host.api('POST', path + '/revoke', {'expectedRevision': row['revision']})
        require(row['revoked'] and not row['enabled'], 'followup cleanup did not revoke')


def run_controls(host, spec, missing):
    run = start_run(host, spec, tool_input(marker='http-cap-approval-' + uuid.uuid4().hex))
    challenge = poll(lambda: pending(host, run), bool, 'Required approval absent')
    if 'approval.read' in missing:
        records = host.api('GET', '/api/uar/runs/' + run + '/tool-approval')
        require(records['durable'] and records['records'], 'actual approval not durably visible')
    decision(host, run, True)
    finish(host, run)
    if 'tool.outcomes' in missing:
        receipts = host.api('GET', '/api/uar/runs/' + run + '/tool-outcomes')['receipts']
        require(receipts and all(row['outcome'] == 'succeeded' for row in receipts), 'approved effect did not succeed')
    if 'checkpoint.list' in missing:
        checkpoints = host.api('GET', '/api/uar/runs/' + run + '/checkpoints')
        require(checkpoints['run_id'] == run, 'checkpoint list identity differs')
    return {'run_id': run, 'approval_id': challenge.get('approval_id') or challenge['approvalId']}


def unsupported(host, identifier):
    """Explicit payloads for known unsupported operations, never generic fallthrough."""
    fake = str(uuid.uuid4())  # Nonexistent resource, never an authority record.
    method, template = CONTRACTS[identifier]
    path = re.sub(r'\{[^}]+\}', fake, template)
    body, expected, principal = None, 503, 'owner'
    if identifier == 'memory.policy.update':
        body = {'enabled': False}
    elif identifier == 'memory.update':
        body = {'content': 'unsupported operation control'}
    elif identifier in ('schedule.create', 'schedule.update'):
        request = {'instanceId': fake, 'prompt': 'unsupported operation control',
                   'recurrence': {'kind': 'once', 'at': (datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(days=1)).isoformat()}}
        body = request if identifier.endswith('create') else {'expectedRevision': 1, 'schedule': request}
    elif identifier in ('schedule.pause', 'schedule.resume', 'schedule.cancel', 'followup.enable', 'followup.disable', 'followup.revoke'):
        body = {'expectedRevision': 1}
    elif identifier in ('followup.create', 'followup.update'):
        request = {'sourceInstanceId': fake, 'destinationInstanceId': fake, 'conversationId': fake, 'eventKinds': ['turn_completed']}
        body = request if identifier.endswith('create') else {'expectedRevision': 1, 'rule': request}
    elif identifier == 'approval.read':
        expected = 200
    elif identifier == 'approval.decide':
        body, expected = {'approval_id': fake, 'approved': False}, 404
    elif identifier == 'checkpoint.resume':
        body = {}
    elif identifier == 'recovery.challenge':
        principal = 'host'
        body = {'runtime_epoch': fake, 'target': {'kind': 'checkpoint_epoch', 'runtime_epoch': fake}}
    elif identifier == 'recovery.exclude':
        principal = 'host'
    value = host.api(method, path, body, expect=expected, principal=principal)
    if identifier == 'approval.read':
        require(value['durable'] is False and value['records'] == [], 'unsupported approval read falsely claimed durability')
    if identifier == 'approval.decide':
        require(value['resolved'] is False, 'unsupported approval decision reported resolution')
    return {'method': method, 'path': path, 'status': expected, 'principal': principal}


def run_http_capabilities(host, spec, evidence):
    inventory = host.api('GET', '/api/uar/assistant-capabilities')
    entries = inventory_http(inventory)
    available = {key for key, entry in entries.items() if entry['available']}
    covered = existing(evidence, host, available)
    missing = {key for key in available if not covered[key]}
    memory_missing = {key for key in missing if key.startswith('memory.')}
    if any(key.startswith('memory.') for key in available):
        # Existing stage covers memory mutations; this supplements its missing
        # positive semantic search and owner export without repeating them.
        memory_controls(host, evidence, memory_missing)
    for prefix, driver in [('schedule.', schedule_controls), ('followup.', followup_controls)]:
        family = {key for key in missing if key.startswith(prefix)}
        if family:
            driver(host, spec, family)
    run_result = None
    if missing & {'tool.outcomes', 'approval.read', 'approval.decide', 'checkpoint.list'}:
        run_result = run_controls(host, spec, missing)
    unavailable = {key: unsupported(host, key) for key, entry in entries.items() if not entry['available']}
    covered = existing(evidence, host, available)
    pending_ops = {key for key in available if not covered[key]}
    require(pending_ops <= PROTECTED, 'ordinary HTTP operation lacks real driver: ' + str(pending_ops))
    evidence.observe('discovered-http-operations', {'inventory': inventory, 'host': host.label,
        'covered': {key: indices for key, indices in covered.items() if indices},
        'unavailable': unavailable, 'pending_protected_operations': sorted(pending_ops),
        'new_required_run': run_result, 'complete': not pending_ops})


def reconcile_http_capabilities(spec, evidence):
    paths = spec.get('capability_receipts', [])
    if not paths:
        raise Blocked('same-inventory actual capability and supervisor receipts required')
    receipts = []
    for path in paths:
        receipt = read(path)
        require(receipt['status'] == 'passed' and receipt['profile'] == evidence.profile, 'capability receipt profile/status mismatch')
        require(receipt['inventory_sha256'] == evidence.identity['inventory_sha256']
                and receipt['source_revision'] == evidence.identity['source_revision'], 'stale capability source receipt')
        require(receipt['artifact'] == evidence.artifact, 'capability receipt executable provenance differs')
        for observation in receipt['observations']:
            require(read(Path(path).parent / (observation['id'] + '.json')) == observation, 'capability observation was changed')
        receipts.append((path, receipt))
    inventories = [(path, receipt, observation['data']) for path, receipt in receipts for observation in receipt['observations']
                   if observation['id'] == 'discovered-http-operations']
    require(inventories, 'actual HTTP discovery stage absent')
    entries = inventory_http(inventories[0][2]['inventory'])
    require(all(inventory_http(item[2]['inventory']) == entries for item in inventories), 'capability availability changed across receipts')
    expected = {key for key, entry in entries.items() if entry['available']}
    covered, unavailable = {}, {}
    for path, receipt, packet in inventories:
        for identifier, indices in packet['covered'].items():
            require(identifier in expected and indices, 'invalid operation coverage packet')
            if identifier in PROTECTED:
                # A successful trace cannot replace physical/continuation evidence.
                continue
            for index in indices:
                row = receipt['http'][index]
                require(row['host'] == packet['host'] and matches(identifier, row), 'operation does not match actual owned successful HTTP request')
            covered[identifier] = {'receipt': str(path), 'receipt_sha256': digest(path), 'http_indices': indices}
        for identifier, result in packet['unavailable'].items():
            require(not entries[identifier]['available'], 'unsupported result used for available operation')
            require(any(all(row.get(key) == result[key] for key in ('method', 'path', 'status', 'principal'))
                        and row.get('host') == packet['host'] for row in receipt['http']), 'unsupported operation lacks actual request')
            unavailable[identifier] = {'receipt': str(path), 'result': result}
    # Qualified fixed supervisor source verifies physical exclusion, then actual
    # competing checkpoint admission and independently observed continuation effects.
    for path, receipt in receipts:
        for observation in receipt['observations']:
            if observation['id'] not in ('supervisor-native-safe-checkpoint', 'supervisor-kubernetes-safe-checkpoint'):
                continue
            data, http = observation['data'], observation['data']['http']
            require(http['challenge_status'] == 200 and http['exclusion_status'] == 200
                    and http['replayed_challenge_status'] == 409, 'supervisor operation did not commit exactly once')
            require(sorted(http['resume_statuses']) == [200, 409] and len(http['resumed_run_ids']) == 1,
                    'protected resume lacked its single real winner')
            independent = data['independent_observations']
            require(len(independent) >= 2 and data['actual_effect_counts']
                    and all(row['continuation_effect_counts'] == data['actual_effect_counts'] for row in independent),
                    'resumed execution lacks independent actual effect observations')
            for identifier, field in [('recovery.challenge', 'challenge_path'), ('recovery.exclude', 'exclusion_path'), ('checkpoint.resume', 'resume_path')]:
                if identifier not in expected:
                    continue
                row = {'method': 'POST', 'path': http[field], 'status': 200,
                       'principal': 'owner' if identifier == 'checkpoint.resume' else 'host'}
                require(matches(identifier, row), 'qualified helper invoked a different protocol route')
                covered[identifier] = {'receipt': str(path), 'receipt_sha256': digest(path),
                    'observation': observation['id'], 'path': http[field]}
    missing = expected - set(covered)
    require(set(unavailable) == set(entries) - expected, 'unavailable HTTP entries lack explicit actual refusal')
    if missing:
        raise Blocked('advertised HTTP operations lack real observations: ' + ', '.join(sorted(missing)))
    evidence.observe('reconciled-http-operations', {'inventory': inventories[0][2]['inventory'],
        'covered': covered, 'unavailable': unavailable, 'exact_available_set': sorted(expected),
        'complete': True, 'publication': 'not_claimed'})
