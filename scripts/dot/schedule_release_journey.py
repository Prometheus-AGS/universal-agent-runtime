"""Real terminal stream release against due scanners under a dedicated VM clock."""
import json
from pathlib import Path
import socket
import time
import urllib.parse
import urllib.request
import uuid

from acceptance import Blocked, Host, digest, poll, require
from calendar_journey import GuestClock, create, get, same_instant, stable, control
from journeys import decision, effects, pending
from provider import Provider


def run_schedule_release(spec, evidence):
    hosts = [Host(row, evidence, 'release-' + name) for name, row in spec['hosts'].items()]
    require(len(hosts) == (1 if evidence.profile == 'local_surreal_kv' else 2), 'release profile host count differs')
    require(all(host.spec['lifecycle'] == 'child_process' for host in hosts), 'clock case requires dedicated guest processes')
    configs = [json.loads(Path(host.spec['config']).read_text()) for host in hosts]
    require(all(row.get('resilience', {}).get('timeout_disabled') is True for row in configs),
            'operator-clock interval requires explicit timeout_disabled on dedicated runtime')
    if len(hosts) == 1:
        require(configs[0]['persistence']['database_url'].startswith('surrealkv://'), 'actual durable KV required')
    else:
        stores = [row['persistence'] for row in configs]
        require(stores[0] == stores[1], 'scanner peers use different durable stores')
        target = urllib.parse.urlparse(stores[0]['database_url'])
        require(target.scheme in ('ws', 'wss', 'http', 'https'), 'actual networked shared store required')
        require({entry[4][0] for entry in socket.getaddrinfo(target.hostname, None)} <= {'127.0.0.1', '::1'},
                'shared database must be in the controlled guest clock domain')
        item = spec['database_provisioning_evidence']
        require(digest(item['path']) == item['sha256'], 'guest database provisioning evidence differs')
    clock = GuestClock(spec['guest_clock'], evidence)
    token = uuid.uuid4().hex
    provider = Provider(spec['provider_port'], evidence.directory / 'schedule-release-provider.jsonl', token)
    provider.start()
    try:
        clock.move('2028-01-01T08:00:00Z')
        for host in hosts:
            host.start()
        host = hosts[0]
        marker = uuid.uuid4().hex
        prompt = json.dumps({'dot_action': 'tool', 'tool': 'effect', 'arguments': {'mode': 'success', 'marker': marker},
                             'marker': marker, 'wire_mode': 'schedule_release_barrier'})
        row, target = create(host, spec, {'kind': 'daily', 'time': '09:00', 'timezone': 'UTC'}, prompt)
        clock.move('2028-01-01T09:00:01Z')
        row = poll(lambda: get(host, row['id']), lambda value: bool(value.get('activeRunId')), 'first due occurrence absent')
        first = row['activeRunId']
        poll(lambda: pending(host, first), bool, 'first Required approval absent')
        require(not effects(spec['effect_trace'], marker), 'initial schedule bypassed approval')
        decision(host, first, True)
        poll(lambda: provider.rows(), lambda rows: any(value.get('event') == 'barrier_waiting' for value in rows),
             'real upstream did not hold terminal usage after the approved effect')
        require(len(effects(spec['effect_trace'], marker)) == 1, 'first external effect not observed exactly once')
        clock.move('2028-01-04T09:00:01Z')
        busy = poll(lambda: get(host, row['id']), lambda value: value['pending'] is not None and
            same_instant(value['pending']['dueAt'], '2028-01-04T09:00:00Z'), 'latest pending occurrence absent')
        require(busy['activeRunId'] == first and not busy['receipts'], 'terminal barrier did not retain overlap guard')
        # Actual wall time is advanced immediately before releasing actual wire
        # completion. Both installed scanners may now race release and catch-up.
        clock.move('2028-01-05T09:00:00Z')
        released_at = time.time()
        request = urllib.request.Request('http://127.0.0.1:' + str(spec['provider_port']) + '/__dot/release',
            data=json.dumps({'marker': marker}).encode(), headers={'Authorization': 'Bearer ' + token,
            'Content-Type': 'application/json'}, method='POST')
        with urllib.request.urlopen(request, timeout=5) as response:
            require(json.load(response)['released'], 'upstream release refused')
        current = poll(lambda: get(host, row['id']), lambda value: value.get('activeRunId') not in (None, first),
                       'safe terminal release did not admit latest occurrence')
        second = current['activeRunId']
        approval = poll(lambda: pending(host, second), bool, 'latest occurrence bypassed Required approval')
        require(same_instant(current['active']['dueAt'], '2028-01-05T09:00:00Z'), 'stale pending occurrence admitted')
        require(current['active']['revision'] == current['revision'], 'admission used stale schedule revision')
        require(len(effects(spec['effect_trace'], marker)) == 1, 'second effect ran before approval')
        stable(hosts, row['id'], lambda value: value['activeRunId'] == second and len(value['receipts']) == 1, clock)
        decision(host, second, True)
        terminal = poll(lambda: get(host, row['id']), lambda value: len(value['receipts']) == 2 and value['active'] is None,
                        'latest occurrence did not release safely')
        terminal = stable(hosts, row['id'], lambda value: len(value['receipts']) == 2 and value['active'] is None, clock)
        require(len(effects(spec['effect_trace'], marker)) == 2, 'racing scanners duplicated external effects')
        require(sum(skip['count'] for skip in terminal['skipped'] if skip['reason'] == 'coalesced') == 3,
                'intermediate January2/3/4 occurrences not accounted as coalesced')
        calls = {run: host.api('GET', '/api/uar/accounting/calls/' + run) for run in (first, second)}
        require(all(rows and all(value['state'] == 'settled' and value['admission']['reserved_tokens'] > 0
                    for value in rows) for rows in calls.values()), 'raced admission bypassed durable budget reservation/settlement')
        requests = [value for value in provider.rows() if value.get('instruction', {}).get('marker') == marker]
        require(len(requests) == 4, 'expected one tool request and one final request per admitted occurrence')
        control(host, terminal, 'cancel')
        evidence.observe('schedule-safe-release-race', {'before': busy, 'after': terminal, 'released_at': released_at,
            'runs': [first, second], 'approval': approval, 'accounting': calls, 'provider_calls': len(requests), 'effect_count': 2})
    finally:
        try:
            for host in reversed(hosts):
                host.stop()
        finally:
            try:
                provider.stop()
            finally:
                clock.restore()
