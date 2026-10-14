"""Real IANA overnight/fold membership and cold durable inbox delivery."""
import json
from pathlib import Path
import socket
import time
import urllib.parse

from acceptance import Blocked, Host, digest, poll, require
from calendar_journey import GuestClock, timestamp
from journeys import instance, segment, turn
from provider import Provider


def run_followup_clock(spec, evidence):
    hosts = [Host(value, evidence, 'followup-' + label) for label, value in spec['hosts'].items()]
    require(len(hosts) == (1 if evidence.profile == 'local_surreal_kv' else 2), 'wrong followup host count')
    stores = [json.loads(Path(host.spec['config']).read_text())['persistence'] for host in hosts]
    if len(hosts) == 1:
        require(stores[0]['database_url'].startswith('surrealkv://'), 'followup local profile needs durable KV')
    else:
        require(stores[0] == stores[1], 'shared followup runtimes need the same actual database')
        address = urllib.parse.urlsplit(stores[0]['database_url']).hostname
        require(address and {row[4][0] for row in socket.getaddrinfo(address, None)} <= {'127.0.0.1', '::1'},
                'clock qualification database must reside in the same isolated guest')
        proof = spec['database_provisioning_evidence']
        require(Path(proof['path']).is_file() and digest(proof['path']) == proof['sha256'], 'guest database proof missing')
        require(all(host.spec['environment'].get('UAR_REMOTE_SURREAL_DURABILITY_ATTESTED') == '1' for host in hosts),
                'shared durable storage not attested')
    clock = GuestClock(spec['guest_clock'], evidence)
    provider = Provider(spec['provider_port'], evidence.directory / 'followup-provider.jsonl')
    provider.start()
    try:
        clock.move('2027-11-07T02:50:00Z')
        for host in hosts:
            host.start()
        host = hosts[0]
        source = instance(host, spec['deployment_binding_id'])
        destination = instance(host, spec['deployment_binding_id'])
        inbox = '/api/uar/followups/inbox/' + segment(destination['sessionId'])
        body = {'sourceInstanceId': source['instanceId'], 'destinationInstanceId': destination['instanceId'],
                'conversationId': destination['sessionId'], 'eventKinds': ['turn_completed'],
                'quietHours': {'start': '22:00', 'end': '07:00', 'timezone': 'America/Chicago'}}
        rules = []
        for _ in range(2):
            rule = host.api('POST', '/api/uar/followups/', body, expect=201)
            rule = host.api('POST', '/api/uar/followups/' + rule['id'] + '/enable', {'expectedRevision': rule['revision']})
            rules.append(rule)
        def get(rule):
            return host.api('GET', '/api/uar/followups/' + rule['id'])
        def items(rule, reader=host):
            return [item for item in reader.api('GET', inbox) if item['ruleId'] == rule['id']]
        def deferred(rule, release):
            row = poll(lambda: get(rule), lambda value: any(entry['disposition'] == 'deferred' for entry in value['decisions']),
                       'committed real source event not deferred')
            decision = next(entry for entry in row['decisions'] if entry['disposition'] == 'deferred')
            require(timestamp(decision['availableAt']) == timestamp(release), 'quiet release instant differs')
            require(not items(rule), 'quiet event delivered before release')
            return row
        clock.move('2027-11-07T03:00:00Z')  # 22:00 CDT, exact quiet start.
        command, terminal = turn(host, source['instanceId'], json.dumps({'dot_action': 'text', 'marker': 'overnight-source'}))
        snapshots = [deferred(rule, '2027-11-07T13:00:00Z') for rule in rules]
        for instant in ['2027-11-07T06:30:00Z', '2027-11-07T07:30:00Z']:
            clock.move(instant)  # Both real instants called 01:30 in Chicago.
            deadline = time.monotonic() + 6
            while time.monotonic() < deadline:
                clock.ensure()
                for rule in rules:
                    deferred(rule, '2027-11-07T13:00:00Z')
                time.sleep(.25)
        edited_body = dict(body, enabled=True,
                           quietHours={'start': '22:00', 'end': '08:00', 'timezone': 'America/Chicago'})
        current = get(rules[1])
        rules[1] = host.api('PUT', '/api/uar/followups/' + current['id'],
                           {'expectedRevision': current['revision'], 'rule': edited_body})
        deferred(rules[1], '2027-11-07T14:00:00Z')
        # Stop every scanner after durable source/deferred records exist and
        # before delivery. Preserve storage; no fabricated outbox is inserted.
        for node in reversed(hosts):
            node.stop(crash=True)
        clock.move('2027-11-07T13:00:00Z')  # Exact original 07:00 permits delivery.
        for node in hosts:
            node.start()
        delivered = poll(lambda: items(rules[0]), bool, 'cold scanner failed committed inbox delivery')
        require(len(delivered) == 1, 'cold recovery duplicated inbox delivery')
        deferred(rules[1], '2027-11-07T14:00:00Z')
        for node in hosts:
            require(items(rules[0], node) == delivered, 'shared inbox disagrees after cold recovery')
        clock.move('2027-11-07T14:00:00Z')
        second = poll(lambda: items(rules[1]), bool, 'edited quiet hours did not release')
        require(len(second) == 1, 'edited rule duplicated delivery')
        for node in reversed(hosts):
            node.stop()
        for node in hosts:
            node.start()
        require(items(rules[0]) == delivered and items(rules[1]) == second, 'delivery replayed after joined restart')
        evidence.observe('followup-overnight-fold-policy', {'source_command': command, 'source_terminal': terminal,
            'deferred': snapshots, 'inbox_original': delivered, 'inbox_edited': second,
            'fold_instants': ['2027-11-07T06:30:00Z', '2027-11-07T07:30:00Z']})
        evidence.observe('followup-cold-inbox-once', {'first_delivery_count': len(delivered), 'second_delivery_count': len(second),
            'interruption': 'all scanner processes killed after durable source/defer and before delivery',
            'external_dispatch': 'not implemented or exercised; no external effect retry claim'})
    finally:
        try:
            for host in reversed(hosts):
                host.stop()
        finally:
            try:
                provider.stop()
            finally:
                clock.restore()
