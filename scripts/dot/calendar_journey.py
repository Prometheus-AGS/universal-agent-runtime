"""Installed schedule worker under a disposable Linux VM's real wall clock.

Called only by the full-phase acceptance entrypoint. No simulated runtime clock,
protected-store writes, arbitrary commands or supplied outcome callbacks exist.
See docs/dot-calendar-qualification.md for the exact isolated-guest contract.
"""
import datetime
import json
from pathlib import Path
import socket
import subprocess
import sys
import time
import urllib.parse
import uuid

from acceptance import ROOT, Blocked, Host, digest, poll, require
from journeys import decision, effects, finish, instance, pending, segment, tool_input
from provider import Provider

BASE = '/api/uar/schedules'


def timestamp(value):
    return datetime.datetime.fromisoformat(value.replace('Z', '+00:00')).timestamp()


def same_instant(actual, expected):
    return timestamp(actual) == timestamp(expected)


class GuestClock:
    def __init__(self, spec, evidence):
        if sys.platform != 'linux' or not hasattr(time, 'clock_settime'):
            raise Blocked('calendar qualification requires a disposable Linux VM')
        if spec.get('allow_guest_clock_changes') is not True:
            raise Blocked('explicit isolated guest clock authorization is missing')
        machine = Path('/etc/machine-id').read_text().strip()
        boot = Path('/proc/sys/kernel/random/boot_id').read_text().strip()
        require(machine == spec['machine_id'] and boot == spec['boot_id'],
                'calendar guest identity differs from authorized disposable VM')
        for name in ['hypervisor_clock_sync_disabled_evidence', 'isolated_guest_evidence']:
            receipt = spec[name]
            if not Path(receipt['path']).is_file():
                raise Blocked('VM isolation prerequisite absent: ' + name)
            require(digest(receipt['path']) == receipt['sha256'], 'VM prerequisite evidence changed')
        vm = subprocess.run(['systemd-detect-virt', '--vm'], capture_output=True, text=True, timeout=10)
        container = subprocess.run(['systemd-detect-virt', '--container'], capture_output=True, text=True, timeout=10)
        if vm.returncode != 0 or container.returncode != 1:
            raise Blocked('calendar clock mutation is forbidden outside a VM or inside a container')
        ntp = subprocess.run(['timedatectl', 'show', '--property=NTP', '--value'],
                             capture_output=True, text=True, timeout=10)
        if ntp.returncode != 0 or ntp.stdout.strip() != 'no':
            raise Blocked('disable guest time synchronization before calendar qualification')
        self.evidence = evidence
        self.initial_wall = time.time()
        self.initial_mono = time.monotonic()
        self.last_wall = None
        self.last_mono = None
        self.changed = False
        self.steps = 0
        zone = Path('/usr/share/zoneinfo/America/Chicago')
        if not zone.is_file():
            raise Blocked('installed Chicago IANA timezone data is unavailable')
        evidence.observe('calendar-guest', {'machine_id': machine, 'boot_id': boot,
            'virtualization': vm.stdout.strip(), 'ntp': ntp.stdout.strip(),
            'timezone_sha256': digest(zone), 'initial_wall': self.initial_wall,
            'prerequisites': {name: spec[name] for name in [
                'hypervisor_clock_sync_disabled_evidence', 'isolated_guest_evidence']}})

    def ensure(self):
        if self.last_wall is not None:
            expected = self.last_wall + time.monotonic() - self.last_mono
            require(abs(time.time() - expected) < 3,
                    'another clock writer changed the guest; calendar evidence is invalid')

    def move(self, instant):
        self.ensure()
        target = timestamp(instant)
        require(self.last_wall is None or target > time.time(), 'scenario must advance monotonically')
        before = time.time()
        time.clock_settime(time.CLOCK_REALTIME, target)
        self.changed = True
        self.last_wall, self.last_mono = target, time.monotonic()
        self.steps += 1
        self.evidence.observe('calendar-clock-' + str(self.steps),
            {'before': before, 'requested': instant, 'observed': time.time()})
        self.ensure()

    def restore(self):
        if self.changed:
            target = self.initial_wall + time.monotonic() - self.initial_mono
            time.clock_settime(time.CLOCK_REALTIME, target)
            self.evidence.observe('calendar-clock-restored', {'requested': target, 'observed': time.time()})


def get(host, id):
    row = host.api('GET', BASE + '/' + segment(id))
    require('prompt' not in row and 'ownerGrant' not in row, 'private schedule authority leaked')
    return row


def create(host, spec, recurrence, prompt):
    target = instance(host, spec['deployment_binding_id'])
    row = host.api('POST', BASE + '/', {'instanceId': target['instanceId'],
        'recurrence': recurrence, 'prompt': prompt}, expect=201)
    return row, target


def control(host, row, action):
    return host.api('POST', BASE + '/' + segment(row['id']) + '/' + action,
                    {'expectedRevision': row['revision']})


def receipt(host, id, approve=False):
    def current():
        row = get(host, id)
        if approve and row.get('activeRunId') and pending(host, row['activeRunId']):
            decision(host, row['activeRunId'], True)
        return row
    row = poll(current, lambda value: bool(value['receipts']), 'schedule did not commit terminal receipt')
    require(len(row['receipts']) == 1 and row['receipts'][0]['disposition'] == 'completed',
            'expected one completed schedule occurrence')
    require(row['active'] is None, 'completed occurrence still holds guard')
    return row


def stable(hosts, id, predicate, clock, seconds=8):
    """Observe at least four actual two-second scanner wakeups using monotonic time."""
    deadline = time.monotonic() + seconds
    last = None
    while time.monotonic() < deadline:
        clock.ensure()
        for host in hosts:
            last = get(host, id)
            require(predicate(last), 'schedule invariant changed: ' + json.dumps(last))
        time.sleep(.25)
    return last


def calendar_cases(hosts, spec, evidence, clock, provider):
    host = hosts[0]
    fixture = json.loads((ROOT / 'tests/fixtures/dot14_calendar_cases.json').read_text())
    starts = {'2027-03-14': ('2027-03-14T07:00:00Z', '2027-03-14T08:40:00Z'),
              '2027-07-01': ('2027-07-01T13:00:00Z', '2027-07-01T14:01:00Z'),
              '2027-11-07': ('2027-11-07T05:30:00Z', '2027-11-07T06:31:00Z'),
              '2027-12-01': ('2027-12-01T14:00:00Z', '2027-12-01T15:01:00Z')}
    for case in sorted(fixture['cases'], key=lambda item: item['localDate']):
        before, after = starts[case['localDate']]
        clock.move(before)
        marker = 'calendar-' + uuid.uuid4().hex
        row, target = create(host, spec, case['recurrence'], json.dumps({'dot_action': 'text', 'marker': marker}))
        clock.move(after)
        if case['expectedUtc'] is None:
            row = poll(lambda: get(host, row['id']),
                lambda value: any(skip['reason'] == case['expectedSkipReason'] for skip in value['skipped']),
                'calendar gap was not recorded by production scanner')
            require(any(case['localDate'] in skip['first'] for skip in row['skipped']), 'wrong gap date')
            row = stable(hosts, row['id'], lambda value: not value['active'] and not value['receipts'], clock)
        else:
            row = receipt(host, row['id'])
            require(same_instant(row['receipts'][0]['occurrence']['dueAt'], case['expectedUtc']),
                    'production UTC instant differs from calendar fixture')
            if case.get('forbiddenUtc'):
                clock.move('2027-11-07T07:31:00Z')
                row = stable(hosts, row['id'], lambda value: len(value['receipts']) == 1 and not value['active'], clock)
                require(not any(same_instant(item['occurrence']['dueAt'], case['forbiddenUtc'])
                                for item in row['receipts']), 'repeated local time ran twice')
        requests = [entry for entry in provider.rows() if entry['instruction'].get('marker') == marker]
        require(len(requests) == (0 if case['expectedUtc'] is None else 1), 'independent model call count differs')
        control(host, get(host, row['id']), 'cancel')
        evidence.observe('calendar-' + case['localDate'], {'case': case, 'status': row,
            'instance_id': target['instanceId'], 'provider_request_count': len(requests)})


def approval_hold(hosts, spec, evidence, clock):
    host = hosts[0]
    clock.move('2027-12-02T08:00:00Z')
    marker = uuid.uuid4().hex
    row, _ = create(host, spec, {'kind': 'once', 'at': '2027-12-02T09:00:00Z'}, tool_input('success', marker))
    clock.move('2027-12-02T09:00:01Z')
    row = poll(lambda: get(host, row['id']), lambda value: bool(value.get('activeRunId')), 'schedule root absent')
    run, active = row['activeRunId'], row['active']
    approval = poll(lambda: pending(host, run), bool, 'scheduled Required approval absent')
    require(not effects(spec['effect_trace'], marker), 'scheduled effect preceded approval')
    for operation in ['pause', 'resume', 'cancel']:
        row = control(host, get(host, row['id']), operation)
        require(row['active'] == active, 'management operation changed admitted occurrence')
    stable(hosts, row['id'], lambda value: value['active'] == active and not value['receipts'], clock)
    require(not effects(spec['effect_trace'], marker), 'management operation bypassed approval')
    decision(host, run, True)
    finish(host, run)
    row = receipt(host, row['id'])
    require(row['cancelled'] and len(effects(spec['effect_trace'], marker)) == 1,
            'cancelled schedule lost admitted occurrence or repeated effect')
    evidence.observe('calendar-approval-hold', {'approval': approval, 'run_id': run,
        'active_before': active, 'terminal': row, 'effect_count': 1})


def unknown_hold(hosts, spec, evidence, clock):
    host = hosts[0]
    clock.move('2027-12-03T08:00:00Z')
    marker = uuid.uuid4().hex
    row, target = create(host, spec, {'kind': 'daily', 'time': '09:00', 'timezone': 'UTC'}, tool_input('unknown', marker))
    clock.move('2027-12-03T09:00:01Z')
    row = poll(lambda: get(host, row['id']), lambda value: bool(value.get('activeRunId')), 'unknown-effect root absent')
    run, active = row['activeRunId'], row['active']
    decision(host, run, True)
    finish(host, run, 'error')
    require(len(effects(spec['effect_trace'], marker)) == 1, 'fixture did not commit actual effect before lost response')
    outcomes = host.api('GET', '/api/uar/runs/' + run + '/tool-outcomes')
    clock.move('2027-12-06T09:01:00Z')
    row = poll(lambda: get(host, row['id']), lambda value: value['pending'] is not None
        and same_instant(value['pending']['dueAt'], '2027-12-06T09:00:00Z'), 'busy calendar did not retain latest due occurrence')
    require(row['active'] == active and not row['receipts'], 'unknown outcome released schedule guard')
    require(sum(skip['count'] for skip in row['skipped'] if skip['reason'] == 'coalesced') == 2,
            'busy catch-up did not coalesce intervening December 4/5 occurrences')
    before_edit = row
    row = control(host, row, 'pause')
    revised = host.api('PUT', BASE + '/' + segment(row['id']), {'expectedRevision': row['revision'],
        'schedule': {'instanceId': target['instanceId'], 'recurrence': {'kind': 'daily', 'time': '10:00', 'timezone': 'UTC'},
                     'prompt': tool_input('success', uuid.uuid4().hex)}})
    require(revised['pending'] is None and revised['active'] == active, 'revision reinterpreted unknown predecessor')
    host.api('POST', BASE + '/' + segment(row['id']) + '/resume', {'expectedRevision': row['revision']}, expect=409)
    row = control(host, revised, 'resume')
    row = control(host, row, 'cancel')
    for node in reversed(hosts):
        node.stop()
    for node in hosts:
        node.start()
    row = stable(hosts, row['id'], lambda value: value['cancelled'] and value['active'] == active
                 and value['pending'] is None and not value['receipts'], clock)
    require(len(effects(spec['effect_trace'], marker)) == 1, 'restart or revision replayed unknown effect')
    require(host.api('GET', '/api/uar/runs/' + run + '/tool-outcomes') == outcomes, 'restart changed effect evidence')
    evidence.observe('calendar-unknown-hold', {'busy': before_edit, 'after_restart': row,
        'run_id': run, 'outcomes': outcomes, 'effect_count': 1, 'joined_restart': True})


def paused_catchup(hosts, spec, evidence, clock):
    host = hosts[0]
    clock.move('2027-12-08T08:00:00Z')
    marker = uuid.uuid4().hex
    row, _ = create(host, spec, {'kind': 'daily', 'time': '09:00', 'timezone': 'UTC'}, tool_input('success', marker))
    row = control(host, row, 'pause')
    clock.move('2027-12-11T09:01:00Z')
    row = poll(lambda: get(host, row['id']), lambda value: value['pending'] is not None
        and same_instant(value['pending']['dueAt'], '2027-12-11T09:00:00Z'), 'paused schedule failed latest-due catch-up')
    require(row['active'] is None and not row['receipts'] and not effects(spec['effect_trace'], marker), 'paused schedule dispatched')
    require(sum(skip['count'] for skip in row['skipped'] if skip['reason'] == 'coalesced') == 3,
            'paused catch-up omitted skipped December 8/9/10 candidates')
    before = row
    control(host, row, 'resume')
    row = receipt(host, row['id'], approve=True)
    require(same_instant(row['receipts'][0]['occurrence']['dueAt'], '2027-12-11T09:00:00Z'), 'resume admitted stale pending work')
    control(host, row, 'cancel')
    stable(hosts, row['id'], lambda value: len(value['receipts']) == 1 and not value['active'], clock)
    require(len(effects(spec['effect_trace'], marker)) == 1, 'catch-up dispatched more than latest occurrence')
    evidence.observe('calendar-paused-catchup', {'before_resume': before, 'terminal': row, 'effect_count': 1})


def run_calendar(config, evidence):
    """config is one complete dedicated profile plus a guest_clock object."""
    hosts = [Host(value, evidence, 'calendar-' + label) for label, value in config['hosts'].items()]
    require(len(hosts) == (1 if evidence.profile == 'local_surreal_kv' else 2), 'calendar profile host count mismatch')
    persistence = [json.loads(Path(host.spec['config']).read_text())['persistence'] for host in hosts]
    if len(hosts) == 1:
        require(persistence[0]['database_url'].startswith('surrealkv://'), 'calendar local profile requires durable KV')
    else:
        require(persistence[0] == persistence[1], 'calendar peers must share exact persistence')
        require(persistence[0]['database_url'].startswith(('ws://', 'wss://', 'http://', 'https://')),
                'calendar shared profile requires networked SurrealDB')
        database_host = urllib.parse.urlparse(persistence[0]['database_url']).hostname
        addresses = {entry[4][0] for entry in socket.getaddrinfo(database_host, None)}
        require(addresses and addresses <= {'127.0.0.1', '::1'},
                'shared calendar database must run in the same guest clock domain')
        database = config['database_provisioning_evidence']
        require(Path(database['path']).is_file() and digest(database['path']) == database['sha256'],
                'shared calendar guest-local database provisioning evidence unavailable')
        require(all(host.spec['environment'].get('UAR_REMOTE_SURREAL_DURABILITY_ATTESTED') == '1' for host in hosts),
                'shared calendar durability not attested')
        if not Path(config['durability_evidence']).is_file():
            raise Blocked('shared calendar storage durability evidence unavailable')
    clock = GuestClock(config['guest_clock'], evidence)
    provider = Provider(config['provider_port'], evidence.directory / 'calendar-provider.jsonl')
    provider.start()
    try:
        # Bindings and credentials are prepared through normal production
        # admission. Every scenario creates its own instance and schedule.
        clock.move('2027-03-13T00:00:00Z')
        for host in hosts:
            host.start()
        calendar_cases(hosts, config, evidence, clock, provider)
        approval_hold(hosts, config, evidence, clock)
        unknown_hold(hosts, config, evidence, clock)
        paused_catchup(hosts, config, evidence, clock)
        clock.ensure()
        evidence.observe('calendar-provider-count', {'requests': len(provider.rows()),
            'claim_scope': 'installed worker, actual guest wall clock, joined runtime restart; no reboot exclusion claim'})
    finally:
        try:
            errors = []
            for host in reversed(hosts):
                try:
                    host.stop()
                except Exception as error:
                    errors.append(str(error))
            if errors:
                raise Blocked('calendar child cleanup unconfirmed: ' + '; '.join(errors))
        finally:
            try:
                provider.stop()
            finally:
                clock.restore()
