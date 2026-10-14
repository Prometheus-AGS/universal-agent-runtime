"""Actual spawn/interaction/teardown contract for Dot19 scenario106.

No child artifact is accepted inline by spawn. The profile names a definition
already installed through the real catalog; the host resolves it independently.
Provider barriers control only upstream final wire, never host state or approval.
"""
import copy
import json
import uuid

from acceptance import poll, require
from journeys import decision, finish, pending, segment, start_run
from mcp_journey import WIRE, current_rows, effects, peer_answer, persisted, run_spec


def interaction_path(run):
    return '/api/uar/runs/' + segment(run) + '/mcp-interactions'


def views(host, run):
    return host.api('GET', interaction_path(run))['interactions']


def lifecycle(host, run, artifact_id):
    snapshot = host.api('GET', '/api/uar/runs/' + segment(run) + '/events')
    require(snapshot['gapReason'] is None, 'child lifecycle observation has a retention gap')
    return [event['data']['lifecycle'] for event in snapshot['events']
            if event['eventName'].startswith('agui.subagent.')
            and event['data']['lifecycle']['artifact_id'] == artifact_id]


def await_input(host, run, approvals, excluded=()):
    def observe():
        current = pending(host, run)
        if current:
            # Each actual Required request is resolved through its real root
            # broker. A delegated child never receives a synthetic approval.
            approval = current.get('approval_id') or current['approvalId']
            require(approval not in approvals, 'resolved approval remained pending')
            approvals.append(decision(host, run, True))
        return [view['record'] for view in views(host, run)
                if view['resolvable'] and view['record']['id'] not in excluded]
    records = poll(observe, bool, 'real root/child elicitation did not become pending')
    require(len(records) == 1, 'unexpected concurrent interaction in fixture root')
    return records[0]


def answer(host, run, record, value):
    committed = host.api('POST', interaction_path(run) + '/' + segment(record['id']), value)
    require(committed['response_committed'] and committed['transport_delivery'] == 'unconfirmed',
            'input commit fabricated transport confirmation')


def upstream(provider, marker):
    return [row for row in provider.rows() if row.get('instruction', {}).get('marker') == marker]


def tool_advertised(names, tool):
    return any(name == tool or name.endswith('__' + tool) for name in names)


def request_trace(entry, record):
    records = [row for row in current_rows(entry) if row['event'] == 'elicitation_request'
               and row['message']['id'] == record['request_id']]
    require(len(records) == 1, 'durable interaction does not identify exactly one real peer request')
    return records[0]


def closed_handler(entry, record, request):
    answers = peer_answer(entry, record)
    require(all(row['message'].get('result', {}).get('action') != 'accept' for row in answers),
            'root teardown accepted the unanswered child input')
    events = current_rows(entry)
    if entry['transport'] == 'stdio':
        closed = [row for row in events if row['event'] == 'stdio_eof' and row['pid'] == request['pid']]
    else:
        closed = [row for row in events if row['event'] == 'http_handler_finished'
                  and row['request_id'] == record['request_id']
                  and row['cause'] in ('client_socket_eof', 'client_socket_error', 'response_written')]
    # Durable cancellation alone is not proof of a stopped transport/handler.
    return {'peer_answers': answers, 'handler_close': closed} if closed else None


def run_mcp_child(host, spec, evidence, peers, provider):
    require(peers is not None and provider is not None, 'real MCP and provider collaborators required')
    require({entry['transport'] for entry in peers.entries} == {'stdio', 'http'},
            'child-root contract requires both real transports')
    wire = json.loads(WIRE.read_text())
    for entry in peers.entries:
        configuration = entry.get('mcp_child')
        require(isinstance(configuration, dict), 'mcp_child root_artifact/child_artifact_id required for ' + entry['name'])
        selected = run_spec(spec, entry)
        selected['artifact'] = copy.deepcopy(configuration['root_artifact'])
        child_id = configuration['child_artifact_id']
        child = host.api('GET', '/api/agents/' + segment(child_id))
        require(child['id'] == child_id and child_id != selected['artifact']['id'], 'persisted child identity differs')
        require('spawn_agent' in selected['artifact']['policy']['tools']['allow'], 'root artifact lacks explicit spawn authorization')
        require(child['policy']['tools']['allow'] == [entry['elicit_tool']],
                'installed child must explicitly narrow tool selection to this elicitation tool')
        before_effects = len(effects(entry))
        before_trace = len(current_rows(entry))
        marker = 'mcp-root-child-' + uuid.uuid4().hex
        child_marker = marker + '-child'
        child_prompt = json.dumps({'dot_action': 'tool_sequence', 'marker': child_marker,
            'steps': [{'tool': entry['elicit_tool'], 'arguments': {'mode': 'form'}}]})
        prompt = json.dumps({'dot_action': 'tool_sequence', 'marker': marker,
            'wire_mode': 'root_completion_barrier', 'steps': [
                {'tool': entry['elicit_tool'], 'arguments': {'mode': 'form'}},
                {'tool': 'spawn_agent', 'arguments': {'artifact_id': child_id,
                    'delegated_prompt': child_prompt, 'task_name': 'mcp-child', 'history_fork': {'mode': 'none'}}}]})
        root_a = start_run(host, selected, prompt)
        approvals_a = []
        original = await_input(host, root_a, approvals_a)
        require(original['server_name'] == entry['server_name'] and original['run_id'] == root_a,
                'original root interaction identity differs')
        answer(host, root_a, original, wire['formAccept'])
        original_answer = poll(lambda: peer_answer(entry, original), bool, 'peer did not receive original root answer')
        require(len(original_answer) == 1 and original_answer[0]['message']['result'] == wire['formAccept'],
                'original root answer was changed or duplicated')
        delegated = await_input(host, root_a, approvals_a, (original['id'],))
        barrier = poll(lambda: provider.barrier_state(marker),
                       lambda value: value and value['waiting'] and not value['released'],
                       'root final provider wire was not held after actual spawn result')
        require(not barrier['terminal_usage_sent'], 'held root already received terminal usage')
        for field in ('owner_key', 'run_id', 'session_id', 'workspace_id', 'channel_id', 'connection_id', 'server_name'):
            require(delegated[field] == original[field], 'child replaced original frozen root binding: ' + field)
        transitions = poll(lambda: lifecycle(host, root_a, child_id),
            lambda values: any(row['status'] == 'running' and row['child_run_id'] for row in values),
            'spawned child has no committed running lifecycle')
        running = next(row for row in reversed(transitions) if row['status'] == 'running')
        require(running['parent_run_id'] == root_a and running['child_run_id'] != root_a,
                'child lineage is not bound to the spawning root')
        require(not views(host, running['child_run_id']), 'child minted an independent interaction channel')
        child_requests = upstream(provider, child_marker)
        require(child_requests, 'no actual child provider request observed')
        require(all(tool_advertised(row['tool_names'], entry['elicit_tool'])
                    and not tool_advertised(row['tool_names'], entry['effect_tool'])
                    and not tool_advertised(row['tool_names'], 'spawn_agent') for row in child_requests),
                'child provider received widened tool bindings')
        parent_requests = upstream(provider, marker)
        require(any(tool_advertised(row['tool_names'], entry['effect_tool'])
                    and tool_advertised(row['tool_names'], 'spawn_agent') for row in parent_requests),
                'parent positive tool-authority control absent')

        root_b = start_run(host, selected, json.dumps({'dot_action': 'tool_sequence', 'marker': marker + '-b',
            'steps': [{'tool': entry['elicit_tool'], 'arguments': {'mode': 'form'}},
                      {'tool': entry['effect_tool'], 'arguments': {}}]}))
        approvals_b = []
        independent = await_input(host, root_b, approvals_b)
        require(independent['owner_key'] == original['owner_key'] and independent['server_name'] == original['server_name'],
                'independent root did not use same verified owner/server')
        require(independent['channel_id'] != original['channel_id']
                and independent['connection_id'] != original['connection_id'], 'two roots shared one interaction transport')
        a_request, b_request = request_trace(entry, delegated), request_trace(entry, independent)
        initializations = [row for row in current_rows(entry)[before_trace:] if row['event'] == 'initialize']
        require(len(initializations) == 2, 'child reconnected instead of inheriting root frozen transport')
        require(all('elicitation' in row['params']['capabilities'] for row in initializations),
                'real clients did not advertise elicitation')
        if entry['transport'] == 'stdio':
            require(a_request['pid'] != b_request['pid'], 'independent roots share one stdio process')
        require(len(effects(entry)) == before_effects, 'effect ran while both inputs were pending')
        require(provider.release(marker)['released'], 'actual root provider barrier was not released')
        terminal_a = finish(host, root_a)
        terminal_child = poll(lambda: lifecycle(host, root_a, child_id),
            lambda values: any(row['child_thread_id'] == running['child_thread_id'] and row['status'] == 'cancelled'
                               for row in values), 'root completion did not join and cancel its child')
        closed = poll(lambda: persisted(host, interaction_path(root_a), delegated['id']),
            lambda view: not view['resolvable'], 'completed root retained a usable child handler')
        require(closed['record']['response'] is None and closed['record']['state'] in ('cancelled', 'disconnected'),
                'teardown fabricated an input answer or merely waited for expiry')
        transport_end = poll(lambda: closed_handler(entry, delegated, a_request), bool,
                             'root completion has no real peer EOF/handler completion observation')
        host.api('POST', interaction_path(root_a) + '/' + segment(delegated['id']), wire['formAccept'], expect=409)
        require(persisted(host, interaction_path(root_b), independent['id'])['resolvable'],
                'root A cleanup revoked root B input')
        require(not closed_handler(entry, independent, b_request), 'root A cleanup stopped root B transport')
        answer(host, root_b, independent, wire['formAccept'])
        delivered_b = poll(lambda: peer_answer(entry, independent), bool, 'B handler lost after A cleanup')
        require(len(delivered_b) == 1 and delivered_b[0]['message']['result'] == wire['formAccept'],
                'independent root answer was changed or duplicated')
        require(len(effects(entry)) == before_effects, 'B input acceptance bypassed separate effect approval')
        approvals_b.append(decision(host, root_b, True))
        require(len(set(approvals_b)) == len(approvals_b) and len(approvals_b) >= 2,
                'B effect reused the elicitation tool approval')
        terminal_b = finish(host, root_b)
        require(len(effects(entry)) == before_effects + 1, 'B did not perform exactly one independently approved effect')
        require(host.api('GET', '/api/agents/' + segment(child_id)) == child, 'child execution mutated its registered artifact')
        evidence.observe('mcp-' + entry['name'] + '-child-root-isolation', {
            'transport': entry['transport'], 'root_a': root_a, 'root_b': root_b,
            'persisted_child_artifact': child, 'running_child': running, 'terminal_child_lifecycle': terminal_child,
            'original_root_interaction': original, 'child_interaction': closed, 'independent_interaction': independent,
            'initializations': initializations, 'root_a_terminal': terminal_a, 'root_b_terminal': terminal_b,
            'root_a_peer_close': transport_end, 'root_b_peer_answer': delivered_b,
            'approvals_a': approvals_a, 'approvals_b': approvals_b, 'effect_delta': 1,
            'child_provider_requests': child_requests, 'root_provider_requests': parent_requests,
            'provider_barrier': marker, 'claim_scope': 'actual host spawn, frozen child bindings and per-root joined cleanup'})
