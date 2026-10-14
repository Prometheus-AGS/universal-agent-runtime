"""Exercise discovered native and MCP memory/schedule/followup operations.

HTTP control setup creates real records; native calls execute via the admitted
model/tool pipeline so full principal context and Required approval are retained.
"""
import datetime
import json
import urllib.error
import urllib.request
import uuid

from acceptance import require, poll
from journeys import helper, start_run, finish, tool_input, instance, turn, segment


class MemoryMcp:
    def __init__(self, host, endpoint):
        self.host, self.endpoint, self.session = host, endpoint, None
        initialized = self.rpc('initialize', {'protocolVersion': '2025-11-25',
            'capabilities': {}, 'clientInfo': {'name': 'dot17-installed-client', 'version': '1'}})
        require(initialized['result']['protocolVersion'] == '2025-11-25', 'memory MCP version differs')
        self.rpc('notifications/initialized', None, notification=True)

    def rpc(self, method, params, notification=False):
        headers = {'Authorization': 'Bearer ' + self.host.credential(),
            'Content-Type': 'application/json', 'Accept': 'application/json, text/event-stream',
            'MCP-Protocol-Version': '2025-11-25'}
        if self.session:
            headers['Mcp-Session-Id'] = self.session
        body = {'jsonrpc': '2.0', 'method': method}
        if not notification:
            body['id'] = uuid.uuid4().hex
        if params is not None:
            body['params'] = params
        request = urllib.request.Request(self.host.base + self.endpoint, headers=headers,
                                         data=json.dumps(body).encode())
        with urllib.request.urlopen(request, timeout=120) as response:
            self.session = response.headers.get('Mcp-Session-Id', self.session)
            if notification:
                require(response.status in (200, 202, 204), 'MCP initialized notification refused')
                return None
            if response.headers.get_content_type() == 'text/event-stream':
                for line in response:
                    if line.startswith(b'data:'):
                        value = json.loads(line[5:])
                        if value.get('id') == body['id']:
                            return value
                raise AssertionError('MCP stream ended before correlated reply')
            return json.load(response)

    def call(self, name, args):
        reply = self.rpc('tools/call', {'name': name, 'arguments': args})
        require('result' in reply and not reply['result'].get('isError', False), 'memory MCP call failed: ' + name)
        return reply['result']


def run_native(host, spec, name, args):
    run = start_run(host, spec, tool_input(tool=name, arguments=args))
    finish(host, run, approve=True)
    receipts = host.api('GET', '/api/uar/runs/' + run + '/tool-outcomes')['receipts']
    require(receipts and all(row['outcome'] == 'succeeded' for row in receipts),
            'native host receipt did not report success: ' + name)
    return run, receipts


def exercise_tools(host, spec, evidence):
    inventory = helper('dot16_capability_inventory.py').inventory(host.base, host.credential())
    available = {(op['id'], entry['transport']): entry for op in inventory['operations']
                 for entry in op['entry_points'] if entry['available'] and entry['transport'] in ('native_tool', 'mcp')}
    observed = {}
    record = host.api('POST', '/api/memory', {'content': 'dot17 native tool record'})['id']
    memory_path = '/api/memory/records/' + segment(record)
    native_memory = {
        'memory.policy.read': {},
        'memory.update': {'memory_id': record, 'content': 'dot17 native update'},
        'memory.history': {'memory_id': record},
        'memory.delete': {'memory_id': record},
    }
    for op, args in native_memory.items():
        entry = available.get((op, 'native_tool'))
        if entry is None:
            continue
        run, receipts = run_native(host, spec, entry['target'], args)
        if op == 'memory.update':
            require(host.api('GET', memory_path)['content'] == args['content'], 'native update did not persist')
        elif op == 'memory.delete':
            host.api('GET', memory_path, expect=404)
        observed[(op, 'native_tool')] = {'run': run, 'receipts': receipts}
    mcp_entries = [(key, entry) for key, entry in available.items() if key[1] == 'mcp']
    if mcp_entries:
        endpoint = mcp_entries[0][1]['target'].split('#')[0]
        client = MemoryMcp(host, endpoint)
        record = host.api('POST', '/api/memory', {'content': 'dot17 MCP record'})['id']
        # Order keeps the record alive until every read/update is observed.
        mcp_args = [('memory.policy.read', {}), ('memory.policy.update', {'enabled': False}),
                    ('memory.read', {'id': record}), ('memory.update', {'id': record, 'content': 'dot17 MCP update'}),
                    ('memory.history', {'id': record}), ('memory.delete', {'id': record})]
        for op, args in mcp_args:
            entry = available.get((op, 'mcp'))
            if entry is None:
                continue
            result = client.call(entry['target'].split('#')[1], args)
            if op == 'memory.update':
                require(host.api('GET', '/api/memory/records/' + segment(record))['content'] == args['content'],
                        'MCP update did not persist')
            if op == 'memory.delete':
                host.api('GET', '/api/memory/records/' + segment(record), expect=404)
            observed[(op, 'mcp')] = {'result': result}
    source = instance(host, spec['deployment_binding_id'])
    destination = instance(host, spec['deployment_binding_id'])
    at = (datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(hours=1)).isoformat()
    schedule_body = {'instanceId': source['instanceId'], 'prompt': 'dot17 discovery schedule',
                     'recurrence': {'kind': 'once', 'at': at}}
    prior_ids = {row['id'] for row in host.api('GET', '/api/uar/schedules/')}
    for op in ['create', 'list', 'read', 'update', 'pause', 'resume', 'cancel']:
        key = ('schedule.' + op, 'native_tool')
        if key not in available:
            continue
        args = {'workspaceId': host.workspace}
        if op in ('create', 'update'):
            args['schedule'] = schedule_body
        if op not in ('create', 'list'):
            current = host.api('GET', '/api/uar/schedules/' + schedule_id)
            args['id'] = schedule_id
            if op != 'read':
                args['expectedRevision'] = current['revision']
        run, receipts = run_native(host, spec, available[key]['target'], args)
        if op == 'create':
            new = [row for row in host.api('GET', '/api/uar/schedules/') if row['id'] not in prior_ids]
            require(len(new) == 1, 'native schedule creation did not create exactly one record')
            schedule_id = new[0]['id']
        if op in ('pause', 'resume', 'cancel'):
            after = host.api('GET', '/api/uar/schedules/' + schedule_id)
            require(after['revision'] > current['revision'], 'native schedule control did not commit')
        observed[key] = {'run': run, 'receipts': receipts}
    follow_body = {'sourceInstanceId': source['instanceId'], 'destinationInstanceId': destination['instanceId'],
        'conversationId': destination['sessionId'], 'eventKinds': ['turn_completed']}
    old_ids = {row['id'] for row in host.api('GET', '/api/uar/followups/')}
    for op in ['create', 'list', 'read', 'update', 'enable', 'inbox', 'acknowledge', 'disable', 'revoke']:
        key = ('followup.' + op, 'native_tool')
        if key not in available:
            continue
        args = {'workspaceId': host.workspace}
        if op in ('create', 'update'):
            args['rule'] = follow_body
        if op in ('inbox', 'acknowledge'):
            args['conversationId'] = destination['sessionId']
            if op == 'acknowledge':
                args['itemId'] = inbox[0]['id']
        elif op not in ('create', 'list'):
            current = host.api('GET', '/api/uar/followups/' + rule_id)
            args['id'] = rule_id
            if op != 'read':
                args['expectedRevision'] = current['revision']
        run, receipts = run_native(host, spec, available[key]['target'], args)
        if op == 'create':
            new = [row for row in host.api('GET', '/api/uar/followups/') if row['id'] not in old_ids]
            require(len(new) == 1, 'native followup creation did not create exactly one record')
            rule_id = new[0]['id']
        if op == 'enable':
            turn(host, source['instanceId'], json.dumps({'dot_action': 'text', 'marker': 'native-followup-event'}))
            inbox = poll(lambda: [item for item in host.api('GET', '/api/uar/followups/inbox/' + segment(destination['sessionId']))
                                  if item['ruleId'] == rule_id], bool, 'native followup source event not delivered')
            require(len(inbox) == 1, 'duplicate native followup delivery')
        observed[key] = {'run': run, 'receipts': receipts}
    require(set(observed) == set(available), 'discovered tool operation lacks a real driver: ' + str(set(available) - set(observed)))
    evidence.observe('discovered-tool-operations', {'entries': [
        {'operation': key[0], 'transport': key[1], 'observation': value} for key, value in observed.items()]})
