"""Installed-host composition journeys. No direct database writes or fake grants."""
import datetime
import importlib.util
import json
import os
from pathlib import Path
import time
import urllib.parse
import uuid

from acceptance import ROOT, Blocked, Host, fixture, poll, require
from provider import Provider, provider_for

INSTANCE = '/api/uar/agent-instances/v1'


def segment(value):
    return urllib.parse.quote(value, safe='')


def helper(name):
    path = ROOT / 'tests/fixtures' / name
    spec = importlib.util.spec_from_file_location(name.removesuffix('.py'), path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def effects(path, marker):
    if isinstance(path, list):
        from kubernetes import read_trace
        rows = [row for reference in path for row in read_trace(reference)]
    elif isinstance(path, dict):
        from kubernetes import read_trace
        rows = read_trace(path)
    else:
        rows = [json.loads(line) for line in Path(path).read_text().splitlines()] if Path(path).exists() else []
    return [row for row in rows if row.get('event') == 'effect' and row.get('marker') == marker]


def tool_input(mode='success', marker=None, tool='effect', arguments=None):
    return json.dumps({'dot_action': 'tool', 'tool': tool,
                      'arguments': arguments if arguments is not None else {
                          'mode': mode, 'marker': marker or uuid.uuid4().hex}})


def start_run(host, spec, prompt):
    body = {'artifact': spec['artifact'], 'input': prompt,
            'run_credentials': spec['run_credentials']}
    return host.api('POST', '/api/uar/runs', body)['run_id']


def pending(host, run):
    value = host.api('GET', '/api/uar/runs/' + run + '/tool-approval/pending')
    return value.get('pending')


def decision(host, run, approved):
    value = poll(lambda: pending(host, run), bool, 'required approval absent')
    approval = value.get('approval_id') or value['approvalId']
    body = {'approval_id': approval, 'approved': approved}
    host.api('POST', '/api/uar/runs/' + run + '/tool-approval', body)
    return approval


def finish(host, run, status='done', approve=False):
    def state():
        if approve and pending(host, run):
            decision(host, run, True)
        return host.api('GET', '/api/uar/runs/' + run)
    value = poll(state, lambda row: row['status'] in ('done', 'error', 'cancelled'), 'run not terminal')
    require(value['status'] == status, 'unexpected terminal run: ' + json.dumps(value))
    return value


def memory(host, peer, spec, evidence):
    marker = 'Dot memory private marker ' + uuid.uuid4().hex
    policy = host.api('GET', '/api/memory/policy')
    require(not policy['enabled'], 'fresh isolated owner must default capture off')
    require(policy['host_capture_enabled'], 'journey config must enable host capture capability')
    host.api('PUT', '/api/memory/policy', {'enabled': True}, expect=400)
    for days in [0, 8, 366]:
        host.api('PUT', '/api/memory/policy', {'enabled': True,
                 'retention': {'kind': 'days', 'days': days}}, expect=400)
    # Explicit memory is distinct from automatic capture consent.
    created = host.api('POST', '/api/memory', {'content': marker})
    key = segment(created['id'])
    require(host.api('GET', '/api/memory/records/' + key)['content'] == marker, 'explicit save lost')
    for principal in ['other_tenant', 'other_issuer']:
        host.api('GET', '/api/memory/records/' + key, principal=principal, expect=(401, 404))
        records = host.api('GET', '/api/memory/records', principal=principal, expect=(200, 401))
        require(marker not in json.dumps(records), 'foreign owner export exposed content')
    host.api('POST', '/api/memory', {'content': marker, 'user_id': 'forged-owner'}, expect=403)
    on = host.api('PUT', '/api/memory/policy', {'enabled': True,
                  'retention': {'kind': 'days', 'days': 365}})
    short = host.api('PUT', '/api/memory/policy', {'enabled': True,
                     'retention': {'kind': 'days', 'days': 7}})
    require(short['revision'] > on['revision'], 'retention change was not revisioned')
    host.api('PATCH', '/api/memory/records/' + key, {'content': marker + ' updated'})
    require(host.api('GET', '/api/memory/records/' + key + '/history')['history'], 'update omitted history')
    off = host.api('PUT', '/api/memory/policy', {'enabled': False})
    require(off['consent_epoch'] > short['consent_epoch'], 'disable did not revoke queued admission epoch')
    control = host.api('GET', '/api/memory/records/' + key + '/control')
    require(control, 'memory control metadata missing')
    host.api('DELETE', '/api/memory/records/' + key)
    host.api('GET', '/api/memory/records/' + key, expect=404)
    host.api('GET', '/api/memory/records/' + key + '/history', expect=(200, 404))
    if peer:
        require(peer.api('GET', '/api/memory/policy')['enabled'] is False, 'peer consent stale')
        peer.api('GET', '/api/memory/records/' + key, expect=404)
    host.restart()
    require(host.api('GET', '/api/memory/policy')['enabled'] is False, 'restart lost consent')
    host.api('GET', '/api/memory/records/' + key, expect=404)
    require(marker not in json.dumps(host.api('GET', '/api/memory/records')), 'deleted content reappeared')
    evidence.observe('memory-owner-controls', {'record_id': created['id'], 'consent_epoch': off['consent_epoch'],
                     'retention_revision': short['revision'], 'joined_restart': True,
                     'claim_scope': 'HTTP owner isolation, explicit update/history/deletion, policy durability'})


def ingestion(host, peer, evidence):
    marker = 'Durable uploaded knowledge ' + uuid.uuid4().hex
    kb = host.api('POST', '/api/uar/knowledge-bases/', {'name': 'dot17-' + uuid.uuid4().hex}, expect=201)
    base = '/api/uar/knowledge-bases/' + segment(kb['id'])
    boundary = 'dot17-' + uuid.uuid4().hex
    source = marker.encode()
    body = (f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="dot17.txt"\r\n'
            'Content-Type: text/plain\r\n\r\n').encode() + source + f'\r\n--{boundary}--\r\n'.encode()
    doc = host.api('POST', base + '/documents', body, expect=202,
                   headers={'Content-Type': 'multipart/form-data; boundary=' + boundary})
    path = base + '/documents/' + segment(doc['id'])
    # Kill after an actually observed 202, preserving immutable accepted bytes.
    # No claim is made about an unobserved earlier worker crash point.
    host.stop(crash=True)
    host.start()
    reader = peer or host
    job = poll(lambda: reader.api('GET', path + '/ingestion'),
               lambda row: row['state'] == 'indexed', 'actual worker failed to index durable upload', 300)
    doc = reader.api('GET', path)
    require(doc['status'].lower() == 'indexed' and doc['chunk_count'] > 0, 'no indexed chunks')
    results = reader.api('POST', base + '/search', {'query': marker, 'limit': 10, 'min_score': 0.0})
    require(any(marker in row['content'] for row in results['results']), 'uploaded bytes absent from worker search')
    for principal in ['other_tenant', 'other_issuer']:
        reader.api('GET', path + '/ingestion', expect=(401, 404), principal=principal)
    reader.api('DELETE', path, expect=(200, 204))
    reader.api('GET', path + '/ingestion', expect=404)
    require(not reader.api('POST', base + '/search', {'query': marker, 'min_score': 0.0})['results'],
            'deletion left searchable chunks')
    evidence.observe('durable-upload-worker', {'document_id': doc['id'], 'chunk_count': doc['chunk_count'],
                     'indexed_generation': job['generation'], 'restart_after_202': True,
                     'publication_reader': reader.label, 'deleted_search_results': 0})


def connected(host, peer, spec, evidence):
    from concurrent.futures import ThreadPoolExecutor
    trace = spec['effect_trace']
    for mode, expected, count in [('success', 'done', 1), ('application_error', 'error', 0), ('unknown', 'error', 1)]:
        marker = uuid.uuid4().hex
        run = start_run(host, spec, tool_input(mode, marker))
        approval = poll(lambda: pending(host, run), bool, 'Required effect approval absent')
        require(len(effects(trace, marker)) == 0, 'effect preceded Required approval')
        approval_id = approval.get('approval_id') or approval['approvalId']
        path = '/api/uar/runs/' + run + '/tool-approval'
        body = {'approval_id': approval_id, 'approved': True}
        # Same decision races through two real runtimes in shared profile.
        with ThreadPoolExecutor(max_workers=2) as pool:
            calls = [pool.submit(node.api, 'POST', path, body, expect=(200, 409)) for node in [host, peer or host]]
            for call in calls:
                call.result()
        finish(host, run, expected)
        receipts = host.api('GET', '/api/uar/runs/' + run + '/tool-outcomes')
        outcomes = receipts['receipts']
        expected_outcome = {'success': 'succeeded', 'application_error': 'application_failed', 'unknown': 'outcome_unknown'}[mode]
        require(outcomes and any(row['outcome'] == expected_outcome for row in outcomes), 'canonical tool outcome differs')
        require(len(effects(trace, marker)) == count, 'independent external effect count differs')
        accounting = host.api('GET', '/api/uar/accounting/calls/' + run)
        require(accounting, 'model calls not durably accounted')
        if mode == 'success':
            require(all(row['state'] == 'settled' for row in accounting), 'finished valid usage remains pending')
        host.restart()
        reread = host.api('GET', '/api/uar/runs/' + run + '/tool-outcomes')
        require(reread == receipts, 'canonical tool outcomes changed after joined process restart')
        require(host.api('GET', '/api/uar/accounting/calls/' + run) == accounting, 'restart changed settlement')
        require(len(effects(trace, marker)) == count, 'restart repeated external effect')
        if mode == 'unknown':
            checkpoints = host.api('GET', '/api/uar/runs/' + run + '/checkpoints')
            for checkpoint in checkpoints.get('checkpoints', []):
                host.api('POST', '/api/uar/runs/' + run + '/resume/' + segment(checkpoint['id']),
                         {'run_credentials': spec['run_credentials']}, expect=(409, 422))
            require(len(effects(trace, marker)) == 1, 'unknown effect replayed')
        evidence.observe('connected-' + mode, {'run_id': run, 'approval_id': approval_id,
                         'effect_count': count, 'outcomes': receipts, 'accounting': accounting})


def instance(host, binding):
    return host.api('POST', INSTANCE + '/', {'deploymentBindingId': binding, 'profile': 'on_demand'}, expect=201)


def turn(host, id, prompt):
    command = uuid.uuid4().hex
    host.api('POST', INSTANCE + '/' + segment(id) + '/turns',
             {'commandId': command, 'prompt': prompt}, expect=(200, 202))
    def current():
        value = host.api('GET', INSTANCE + '/' + segment(id))
        run = value.get('activeRunId')
        if run and pending(host, run):
            decision(host, run, True)
        return value
    value = poll(current, lambda row: any(c['commandId'] == command and c['status'] in
                 ('completed', 'failed', 'uncertain', 'cancelled') for c in row['commands']),
                 'logical instance did not finish source turn')
    require(next(c for c in value['commands'] if c['commandId'] == command)['status'] == 'completed',
            'logical source turn failed')
    return command, value


def schedules_followups(host, peer, spec, evidence):
    # Bindings must have been admitted through compiler/deployment production APIs.
    # Their existence is a prerequisite, never forged in protected storage here.
    source = instance(host, spec['deployment_binding_id'])
    destination = instance(host, spec['deployment_binding_id'])
    sid, did = source['instanceId'], destination['instanceId']
    conversation = destination['sessionId']
    follow = '/api/uar/followups'
    rule_body = {'sourceInstanceId': sid, 'destinationInstanceId': did,
        'conversationId': conversation, 'eventKinds': ['turn_completed']}
    rule = host.api('POST', follow + '/', rule_body, expect=201)
    rid = rule['id']
    require(not rule['enabled'], 'followup consent must default off')
    rule = host.api('POST', follow + '/' + rid + '/enable', {'expectedRevision': rule['revision']})
    command, terminal = turn(host, sid, json.dumps({'dot_action': 'text', 'marker': 'source-event-' + uuid.uuid4().hex}))
    inbox_path = follow + '/inbox/' + segment(conversation)
    def own_items(node=host):
        return [row for row in node.api('GET', inbox_path) if row['ruleId'] == rid]
    items = poll(own_items, bool, 'committed source occurrence was not delivered to inbox')
    require(len(items) == 1, 'one source turn produced duplicate deliveries')
    if peer:
        require(own_items(peer) == items, 'shared runtime inbox diverged')
    host.restart()
    require(own_items() == items, 'cold restart lost or duplicated logical inbox delivery')
    host.api('POST', inbox_path + '/' + segment(items[0]['id']) + '/acknowledge', expect=204)
    first_ack = own_items()[0]['acknowledgedAt']
    host.api('POST', inbox_path + '/' + segment(items[0]['id']) + '/acknowledge', expect=204)
    require(own_items()[0]['acknowledgedAt'] == first_ack, 'acknowledgement not idempotent')
    rule = host.api('GET', follow + '/' + rid)
    now = datetime.datetime.now(datetime.timezone.utc)
    quiet = {'start': (now - datetime.timedelta(minutes=1)).strftime('%H:%M'),
             'end': (now + datetime.timedelta(minutes=10)).strftime('%H:%M'), 'timezone': 'UTC'}
    rule_body.update(enabled=True, quietHours=quiet)
    rule = host.api('PUT', follow + '/' + rid, {'expectedRevision': rule['revision'], 'rule': rule_body})
    quiet_command, _ = turn(host, sid, json.dumps({'dot_action': 'text', 'marker': 'quiet-source'}))
    rule = poll(lambda: host.api('GET', follow + '/' + rid),
                lambda row: any(d['disposition'] == 'deferred' for d in row['decisions']), 'quiet event not deferred')
    require(len(own_items()) == 1, 'quiet followup delivered before release')
    revoked = host.api('POST', follow + '/' + rid + '/revoke', {'expectedRevision': rule['revision']})
    require(not own_items(), 'revocation still exposes delivered metadata')
    require(all(d['disposition'] not in ('queued', 'deferred') for d in revoked['decisions']),
            'revocation retained deliverable pending work')
    at = (datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(seconds=15)).isoformat()
    schedule = host.api('POST', '/api/uar/schedules/', {'instanceId': sid,
        'prompt': json.dumps({'dot_action': 'text', 'marker': 'scheduled-source'}),
        'recurrence': {'kind': 'once', 'at': at}}, expect=201)
    schedule = poll(lambda: host.api('GET', '/api/uar/schedules/' + schedule['id']),
                    lambda row: bool(row['receipts']), 'scheduled real instance turn failed', 240)
    require(len(schedule['receipts']) == 1, 'one due occurrence produced duplicate receipts')
    observed = host.api('GET', INSTANCE + '/' + sid)
    require(len(observed['commands']) > len(terminal['commands']), 'schedule receipt without actual new command')
    if peer:
        require(peer.api('GET', '/api/uar/schedules/' + schedule['id'])['receipts'] == schedule['receipts'],
                'schedule receipt not shared')
    evidence.observe('logical-schedule-followup', {'source_instance': sid, 'destination_instance': did,
        'source_command': command, 'quiet_command': quiet_command, 'inbox_delivery_count': len(items),
        'schedule': schedule, 'revocation': revoked, 'joined_restart': True})


def protocols(host, peer, spec, evidence):
    canonical = host.api('GET', '/.well-known/agent-card.json', principal=None)
    alias = host.api('GET', '/.well-known/agent.json', principal=None)
    require(canonical == alias, 'legacy discovery alias differs from canonical card')
    endpoint = '/a2a/v1/compiler' if spec['a2a_agent'] == 'compiler-agent' else '/a2a/v1/agents/' + segment(spec['a2a_agent'])
    def a2a_task(key):
        return host.api('POST', endpoint, {'jsonrpc': '2.0', 'id': uuid.uuid4().hex, 'method': 'GetTask',
            'params': {'id': key}}, headers={'A2A-Version': '1.0'})['result']
    a2a_marker = uuid.uuid4().hex
    env = {'UAR_A2A_BASE_URL': host.base, 'UAR_A2A_DISCOVERY_BASE_URL': spec.get('discovery_base_url', host.base), 'UAR_A2A_TEST_TOKEN': host.credential(),
           'UAR_A2A_OTHER_TOKEN': host.credential('other_tenant'),
           'UAR_A2A_AGENT_ID': spec['a2a_agent'],
           'UAR_A2A_INPUT': tool_input('success', a2a_marker)}
    receipt = str(evidence.directory / 'a2a-task.json')
    fixture(host, 'dot18_a2a_real_path.py', ['submit', receipt], env)
    first_task = json.loads(Path(receipt).read_text())['task_id']
    poll(lambda: a2a_task(first_task), lambda task: task['status']['state'] == 'TASK_STATE_INPUT_REQUIRED', 'A2A approval did not become pending')
    require(not effects(spec['effect_trace'], a2a_marker), 'A2A effect preceded approval')
    fixture(host, 'dot18_a2a_real_path.py', ['pending', receipt], env)
    host.restart()
    if peer:
        env['UAR_A2A_BASE_URL'] = peer.base
    fixture(host, 'dot18_a2a_real_path.py', ['read', receipt], env)
    fixture(host, 'dot18_a2a_real_path.py', ['approve', receipt], env)
    poll(lambda: a2a_task(first_task), lambda task: task['status']['state'] == 'TASK_STATE_COMPLETED', 'A2A approved task did not complete')
    require(len(effects(spec['effect_trace'], a2a_marker)) == 1, 'A2A approved task effect count differs')
    evidence.observe('a2a-task-approval', {'task': json.loads(Path(receipt).read_text()), 'joined_restart': True})
    denied_marker = uuid.uuid4().hex
    env['UAR_A2A_INPUT'] = tool_input('success', denied_marker)
    env['UAR_A2A_BASE_URL'] = host.base
    deny_receipt = str(evidence.directory / 'a2a-denied-task.json')
    fixture(host, 'dot18_a2a_real_path.py', ['submit', deny_receipt], env)
    denied_task = json.loads(Path(deny_receipt).read_text())['task_id']
    poll(lambda: a2a_task(denied_task), lambda task: task['status']['state'] == 'TASK_STATE_INPUT_REQUIRED', 'A2A denial task not pending')
    fixture(host, 'dot18_a2a_real_path.py', ['deny', deny_receipt], env)
    rejected = poll(lambda: a2a_task(denied_task), lambda task: task['status']['state'] in (
        'TASK_STATE_REJECTED', 'TASK_STATE_FAILED', 'TASK_STATE_CANCELED'), 'denied task reported no truthful terminal outcome')
    require(not effects(spec['effect_trace'], denied_marker), 'denied A2A task executed effect')
    evidence.observe('a2a-task-denial', {'task': rejected, 'effect_count': 0})
    from concurrent.futures import ThreadPoolExecutor
    race_marker = uuid.uuid4().hex
    endpoint = '/a2a/v1/compiler' if spec['a2a_agent'] == 'compiler-agent' else '/a2a/v1/agents/' + segment(spec['a2a_agent'])
    message = {'jsonrpc': '2.0', 'id': uuid.uuid4().hex, 'method': 'SendMessage', 'params': {
        'message': {'messageId': uuid.uuid4().hex, 'role': 'ROLE_USER',
                    'parts': [{'text': tool_input('success', race_marker)}]},
        'configuration': {'returnImmediately': True}}}
    with ThreadPoolExecutor(max_workers=2) as pool:
        futures = [pool.submit(node.api, 'POST', endpoint, message, headers={'A2A-Version': '1.0'}) for node in [host, peer or host]]
        tasks = [future.result()['result']['task'] for future in futures]
    require(tasks[0]['id'] == tasks[1]['id'], 'competing A2A message created two durable tasks')
    def task_state():
        return host.api('POST', endpoint, {'jsonrpc': '2.0', 'id': uuid.uuid4().hex, 'method': 'GetTask',
            'params': {'id': tasks[0]['id']}}, headers={'A2A-Version': '1.0'})['result']
    waiting = poll(task_state, lambda task: task['status']['state'] == 'TASK_STATE_INPUT_REQUIRED', 'raced task not pending')
    require(not effects(spec['effect_trace'], race_marker), 'raced A2A tool bypassed approval')
    challenge = next(row['challengeId'] for row in waiting['metadata']['uar.approvals'] if row['state'] == 'pending')
    host.api('POST', endpoint, {'jsonrpc': '2.0', 'id': uuid.uuid4().hex, 'method': 'SendMessage', 'params': {
        'message': {'messageId': uuid.uuid4().hex, 'taskId': waiting['id'], 'contextId': waiting['contextId'],
                    'role': 'ROLE_USER', 'parts': [{'text': 'approve'}]},
        'metadata': {'uar.approval': {'id': challenge, 'approved': True}},
        'configuration': {'returnImmediately': True}}}, headers={'A2A-Version': '1.0'})
    terminal_task = poll(task_state, lambda task: task['status']['state'] == 'TASK_STATE_COMPLETED', 'raced A2A task did not complete')
    require(len(effects(spec['effect_trace'], race_marker)) == 1, 'raced A2A task repeated independent effect')
    evidence.observe('a2a-competing-submissions', {'task': terminal_task['id'], 'effect_count': 1})
    marker = uuid.uuid4().hex
    env = {'UAR_URL': host.base, 'UAR_TOKEN': host.credential(), 'UAR_AGENT': spec['agui_agent'],
           'UAR_WORKSPACE': host.workspace}
    thread, run, resumed = [uuid.uuid4().hex for _ in range(3)]
    output = fixture(host, 'dot19_agui_real_path.py', ['submit', thread, run, tool_input('success', marker)], env)
    events = [json.loads(line) for line in output.splitlines()]
    terminal = next(event for event in events if event['type'] == 'RUN_FINISHED')
    require(terminal['outcome']['type'] == 'interrupt', 'Required tool did not interrupt AGUI')
    interrupts = terminal['outcome']['interrupts']
    require(len(interrupts) == 1 and not effects(spec['effect_trace'], marker), 'effect preceded AGUI decision')
    agui_path = '/api/uar/ag-ui/agents/' + segment(spec['agui_agent'])
    invalid = {'threadId': thread, 'runId': uuid.uuid4().hex, 'parentRunId': run, 'messages': [],
               'resume': [{'interruptId': 'not-the-current-interrupt', 'status': 'resolved', 'payload': {'approved': True}}]}
    host.api('POST', agui_path, invalid, expect=409)
    host.api('POST', agui_path, invalid, principal='other_tenant', expect=(403, 404, 409))
    require(not effects(spec['effect_trace'], marker), 'invalid resume caused effect')
    host.restart()
    output = fixture(host, 'dot19_agui_real_path.py', ['resume', thread, resumed, run, interrupts[0]['id'], 'approve'], env)
    require(len(effects(spec['effect_trace'], marker)) == 1, 'AGUI resume did not produce exactly one effect')
    invalid['runId'] = uuid.uuid4().hex
    invalid['resume'][0]['interruptId'] = interrupts[0]['id']
    host.api('POST', agui_path, invalid, expect=409)
    require(len(effects(spec['effect_trace'], marker)) == 1, 'stale resume repeated effect')
    evidence.observe('agui-interrupt-resume', {'thread': thread, 'run': run, 'resume': resumed,
        'invalid_and_stale_resume_denied': True, 'interrupt': interrupts[0]['id'], 'events': [json.loads(line) for line in output.splitlines()], 'effect_count': 1})


def settings(host, spec, evidence):
    mod = helper('dot20_effective_settings.py')
    secrets = [os.environ[name] for name in spec['secret_marker_environment']]
    effective = mod.inspect_installed(host.base, host.credential('admin'), host.credential(),
                                      spec['effective_settings_expected'], secrets)
    for origin in ['https://dot-unlisted.invalid', 'null']:
        _, headers = host.request('GET', '/.well-known/agent-card.json', principal=None,
                                   headers={'Origin': origin})
        require(not any(key.lower() == 'access-control-allow-origin' for key in headers), 'unlisted origin allowed')
    host.api('GET', '/api/uar/runs', principal=None, expect=401)
    if evidence.artifact['kind'] == 'independently_installed_publication':
        card = host.api('GET', '/.well-known/agent-card.json', principal=None)
        require(card['version'] == evidence.artifact['tag'].removeprefix('v'), 'card version differs from published release')
    inventory = helper('dot16_capability_inventory.py').inventory(host.base, host.credential())
    evidence.observe('effective-settings-auth', {'effective': effective, 'private_anonymous_status': 401,
                                                'inventory': inventory})


def run_profile(profile, spec, evidence):
    hosts = [Host(value, evidence, label) for label, value in spec['hosts'].items()]
    require(len(hosts) == (1 if profile == 'local_surreal_kv' else 2), 'profile runtime count mismatch')
    configs = [json.loads(Path(host.spec['config']).read_text()) for host in hosts]
    persistence = [value['persistence'] for value in configs]
    if profile == 'local_surreal_kv':
        require(persistence[0]['database_url'].startswith('surrealkv://'), 'local journey requires real durable KV')
    else:
        require(persistence[0] == persistence[1], 'shared runtimes must use exact same durable database')
        require(persistence[0]['database_url'].startswith(('ws://', 'wss://', 'http://', 'https://')),
                'shared journey requires networked SurrealDB')
        require(all(host.spec['lifecycle'] == 'kubernetes' for host in hosts),
                'normal shared journey requires two actual installed know-me runtime Pods')
        require(hosts[0].spec['kubernetes']['pod'] != hosts[1].spec['kubernetes']['pod'], 'shared journey needs distinct Pod identities')
        require(isinstance(spec['effect_trace'], list) and len(spec['effect_trace']) == 2 and
                {row['pod'] for row in spec['effect_trace']} == {node.spec['kubernetes']['pod'] for node in hosts},
                'independent effect counts must cover both actual runtime Pods')
        require(all(host.spec['environment'].get('UAR_REMOTE_SURREAL_DURABILITY_ATTESTED') == '1' for host in hosts),
                'remote durability not attested')
        if not Path(spec['durability_evidence']).is_file():
            raise Blocked('actual shared storage durability evidence is unavailable')
        memory_configs = [value['memory'] for value in configs]
        require(memory_configs[0]['surreal_endpoint'] == memory_configs[1]['surreal_endpoint'], 'memory stores differ')
        require(memory_configs[0]['namespace'] == memory_configs[1]['namespace'] and
                memory_configs[0]['database'] == memory_configs[1]['database'], 'memory namespaces differ')
    provider = provider_for(spec, evidence)
    provider.start()
    from mcp_journey import prepare_peers, run_mcp
    peers = None
    try:
        peers = prepare_peers(spec, evidence)
        for host in hosts:
            host.start()
        host, peer = hosts[0], hosts[1] if len(hosts) == 2 else None
        settings(host, spec, evidence)
        from credentials_journey import run_credentials
        run_credentials(host, peer, spec, evidence)
        from budget_journey import run_budget
        run_budget(host, peer, spec, evidence, provider)
        memory(host, peer, spec, evidence)
        from owner_isolation_journey import run_owner_isolation, run_unresolved_knowledge
        run_owner_isolation(host, spec, evidence)
        run_unresolved_knowledge(host, spec, evidence, provider)
        connected(host, peer, spec, evidence)
        schedules_followups(host, peer, spec, evidence)
        protocols(host, peer, spec, evidence)
        from capabilities import exercise_tools
        exercise_tools(host, spec, evidence)
        from http_capability_journey import run_http_capabilities
        run_http_capabilities(host, spec, evidence)
        from mcp_child_journey import run_mcp_child
        run_mcp_child(host, spec, evidence, peers, provider)
        run_mcp(host, spec, evidence, peers)
        ingestion(host, peer, evidence)
        evidence.observe('provider-independent-count', {'requests': len(provider.rows())})
    finally:
        for host in reversed(hosts):
            host.stop()
        if peers is not None:
            peers.close()
        provider.stop()
