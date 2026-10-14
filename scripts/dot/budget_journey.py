"""Actual route admission and durable settlement with independent upstream counts."""
from concurrent.futures import ThreadPoolExecutor
import json
import uuid

from acceptance import poll, require
from journeys import finish, start_run


def run_budget(host, peer, spec, evidence, provider):
    path = '/api/uar/accounting/scopes/global/global'
    run = start_run(host, spec, json.dumps({'dot_action': 'text', 'marker': uuid.uuid4().hex}))
    finish(host, run)
    calls = host.api('GET', '/api/uar/accounting/calls/' + run)
    require(calls and all(row['state'] == 'settled' for row in calls), 'initial real call not settled')
    reservation = calls[0]['admission']['reserved_tokens']
    require(isinstance(reservation, int) and reservation > 5, 'selected catalog model lacks a bounded token reservation')
    before = host.api('GET', path, principal='host')
    require(before['pending_calls'] == 0 and before['unknown_calls'] == 0, 'dedicated budget scope has existing liabilities')

    def limit(tokens):
        current = host.api('GET', path, principal='host')
        return host.api('PUT', path, {'expected_generation': current['generation'],
            'limits': {'tokens': tokens, 'cost_usd': before['limits']['cost_usd']},
            'evidence': 'Dot acceptance isolated deployment budget change'}, principal='host')

    try:
        body = {'expected_generation': before['generation'], 'limits': {'tokens': 0, 'cost_usd': None},
                'evidence': 'ordinary principal must not administer deployment budget'}
        host.api('PUT', path, body, expect=403)
        limit(before['settled_tokens'])
        count = len(provider.rows())
        denied = start_run(host, spec, json.dumps({'dot_action': 'text', 'marker': uuid.uuid4().hex}))
        finish(host, denied, 'error')
        require(len(provider.rows()) == count, 'exhausted budget reached provider')
        host.restart()
        require(host.api('GET', path, principal='host')['settled_tokens'] == before['settled_tokens'],
                'restart changed settled spend')
        denied_cold = start_run(peer or host, spec, json.dumps({'dot_action': 'text', 'marker': uuid.uuid4().hex}))
        finish(peer or host, denied_cold, 'error')
        require(len(provider.rows()) == count, 'restart or peer bypassed durable budget')
        limit(before['settled_tokens'] + reservation)
        markers = [uuid.uuid4().hex, uuid.uuid4().hex]
        def launch(node, marker):
            return node, start_run(node, spec, json.dumps({'dot_action': 'text', 'marker': marker, 'delay_seconds': 3}))
        with ThreadPoolExecutor(max_workers=2) as pool:
            futures = [pool.submit(launch, node, marker) for node, marker in zip([host, peer or host], markers)]
            runs = [future.result() for future in futures]
        statuses = []
        for node, key in runs:
            state = poll(lambda: node.api('GET', '/api/uar/runs/' + key),
                         lambda row: row['status'] in ('done', 'error', 'cancelled'), 'budget race did not terminate')
            statuses.append(state['status'])
        require(sorted(statuses) == ['done', 'error'], 'budget race did not admit exactly one model call')
        observed = [row for row in provider.rows() if row['instruction'].get('marker') in markers]
        require(len(observed) == 1, 'independent provider count differs from one admitted call')
        after = host.api('GET', path, principal='host')
        require(after['settled_tokens'] == before['settled_tokens'] + 5 and after['pending_calls'] == 0,
                'winning usage not settled exactly once')
        host.restart()
        require((peer or host).api('GET', path, principal='host') == after, 'settlement changed after joined restart')
        evidence.observe('route-budget-admission', {'initial_run': run, 'denied_runs': [denied, denied_cold],
            'raced_runs': [key for _, key in runs], 'statuses': statuses, 'provider_calls': len(observed),
            'before': before, 'after': after, 'settled_usage_tokens': 5})
    finally:
        limit(before['limits']['tokens'])


def run_streams(host, spec, evidence, provider):
    records = []
    global_path = '/api/uar/accounting/scopes/global/global'
    original_limits = None
    for mode, expected in [('fragmented', 'done'), ('missing_terminal', 'error')]:
        if mode == 'missing_terminal':
            initial = host.api('GET', global_path, principal='host')
            original_limits = initial['limits']
            settled_calls = host.api('GET', '/api/uar/accounting/calls/' + records[0]['run'])
            reservation = settled_calls[0]['admission']['reserved_tokens']
            require(isinstance(reservation, int) and reservation > 5, 'unknown-spend case requires bounded catalog model')
            host.api('PUT', global_path, {'expected_generation': initial['generation'],
                'limits': {'tokens': initial['settled_tokens'] + reservation, 'cost_usd': original_limits['cost_usd']},
                'evidence': 'isolated stream qualification liability boundary'}, principal='host')
        marker = 'Dot streamed multibyte π 🦀 ' + uuid.uuid4().hex
        run = start_run(host, spec, json.dumps({'dot_action': 'text', 'marker': marker, 'wire_mode': mode}))
        finish(host, run, expected)
        events = host.api('GET', '/api/uar/runs/' + run + '/events?after=0')
        rows = [row for row in provider.rows() if row['instruction'].get('marker') == marker]
        require(len(rows) == 1, 'provider retry changed incomplete-stream disposition')
        if mode == 'fragmented':
            require(marker in json.dumps(events, ensure_ascii=False), 'valid fragmented Unicode was changed')
        if mode == 'missing_terminal':
            pending_calls = host.api('GET', '/api/uar/accounting/calls/' + run)
            require(pending_calls and any(row['state'] == 'pending' for row in pending_calls), 'incomplete stream settled unknown usage')
            before_restart = host.api('GET', global_path, principal='host')
            require(before_restart['pending_calls'] > 0, 'incomplete call released durable reservation')
            host.restart()
            require(host.api('GET', global_path, principal='host') == before_restart, 'cold restart forgot unknown liability')
            count = len(provider.rows())
            refused = start_run(host, spec, json.dumps({'dot_action': 'text', 'marker': uuid.uuid4().hex}))
            finish(host, refused, 'error')
            require(len(provider.rows()) == count, 'cold unknown liability permitted another provider dispatch')
            evidence.observe('unknown-spend-cold-admission', {'incomplete_run': run, 'refused_run': refused,
                'scope': before_restart, 'calls': pending_calls, 'new_provider_calls': 0,
                'disposition': 'liability retained; no fabricated provider reconciliation'})
        records.append({'run': run, 'wire_mode': mode, 'terminal': expected, 'provider_calls': len(rows)})
    evidence.observe('installed-stream-terminal', {'cases': records})


def run_stream_profile(spec, evidence):
    from acceptance import Host
    from provider import Provider, provider_for
    hosts = [Host(value, evidence, label) for label, value in spec['hosts'].items()]
    provider = provider_for(spec, evidence)
    provider.start()
    try:
        for host in hosts:
            host.start()
        run_streams(hosts[0], spec, evidence, provider)
    finally:
        for host in reversed(hosts):
            host.stop()
        provider.stop()
